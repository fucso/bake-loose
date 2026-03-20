//! PgTrialRepository 実装

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::parameter::Parameter;
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;
use crate::ports::trial_repository::{TrialRepository, TrialSort};

use super::executor::PgExecutor;
use super::models::trial_row::{trial_status_to_str, ParameterRow, StepRow, TrialRow};

/// PostgreSQL 用の TrialRepository 実装
///
/// Trial を aggregate root として Steps と Parameters を含めて操作する。
/// `PgExecutor` を使用して、pool 直接またはトランザクション内のどちらでも動作する。
#[derive(Clone)]
pub struct PgTrialRepository {
    executor: PgExecutor,
}

impl PgTrialRepository {
    /// 新しい PgTrialRepository を作成する
    pub fn new(executor: PgExecutor) -> Self {
        Self { executor }
    }

    /// Trial ID に紐づく Steps を取得し、各 Step に Parameters を組み立てる
    async fn fetch_steps_with_parameters(
        &self,
        trial_id: Uuid,
    ) -> Result<Vec<Step>, RepositoryError> {
        // Steps を position 順で取得
        let step_rows = self
            .executor
            .fetch_all(
                sqlx::query_as::<_, StepRow>(
                    "SELECT * FROM steps WHERE trial_id = $1 ORDER BY position ASC",
                )
                .bind(trial_id),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        if step_rows.is_empty() {
            return Ok(Vec::new());
        }

        // 全 Step の ID を集めて、Parameters を一括取得（N+1 回避）
        let step_ids: Vec<Uuid> = step_rows.iter().map(|s| s.id).collect();
        let param_rows = self
            .executor
            .fetch_all(
                sqlx::query_as::<_, ParameterRow>(
                    "SELECT * FROM parameters WHERE step_id = ANY($1)",
                )
                .bind(&step_ids),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        // Step ID ごとに Parameters をグループ化
        let mut params_by_step: std::collections::HashMap<Uuid, Vec<Parameter>> =
            std::collections::HashMap::new();
        for row in param_rows {
            let step_id = row.step_id;
            params_by_step
                .entry(step_id)
                .or_default()
                .push(Parameter::from(row));
        }

        // Steps に Parameters を組み立てる
        let steps = step_rows
            .into_iter()
            .map(|row| {
                let params = params_by_step.remove(&row.id).unwrap_or_default();
                row.into_step(params)
            })
            .collect();

        Ok(steps)
    }
}

#[async_trait]
impl TrialRepository for PgTrialRepository {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        let trial_row = self
            .executor
            .fetch_optional(
                sqlx::query_as::<_, TrialRow>("SELECT * FROM trials WHERE id = $1").bind(id.0),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        let Some(trial_row) = trial_row else {
            return Ok(None);
        };

        let steps = self.fetch_steps_with_parameters(trial_row.id).await?;
        Ok(Some(trial_row.into_trial(steps)))
    }

    async fn find_by_project_id(
        &self,
        project_id: &ProjectId,
        sort: TrialSort,
    ) -> Result<Vec<Trial>, RepositoryError> {
        let sql = format!(
            "SELECT * FROM trials WHERE project_id = $1 {}",
            sort.to_order_by_clause()
        );
        let trial_rows = self
            .executor
            .fetch_all(sqlx::query_as::<_, TrialRow>(&sql).bind(project_id.0))
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        if trial_rows.is_empty() {
            return Ok(Vec::new());
        }

        // 全 Trial の ID を集めて Steps を一括取得（N+1 回避）
        let trial_ids: Vec<Uuid> = trial_rows.iter().map(|t| t.id).collect();

        let step_rows = self
            .executor
            .fetch_all(
                sqlx::query_as::<_, StepRow>(
                    "SELECT * FROM steps WHERE trial_id = ANY($1) ORDER BY position ASC",
                )
                .bind(&trial_ids),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        // Step ID を集めて Parameters を一括取得
        let step_ids: Vec<Uuid> = step_rows.iter().map(|s| s.id).collect();
        let param_rows = if step_ids.is_empty() {
            Vec::new()
        } else {
            self.executor
                .fetch_all(
                    sqlx::query_as::<_, ParameterRow>(
                        "SELECT * FROM parameters WHERE step_id = ANY($1)",
                    )
                    .bind(&step_ids),
                )
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?
        };

        // Step ID ごとに Parameters をグループ化
        let mut params_by_step: std::collections::HashMap<Uuid, Vec<Parameter>> =
            std::collections::HashMap::new();
        for row in param_rows {
            let step_id = row.step_id;
            params_by_step
                .entry(step_id)
                .or_default()
                .push(Parameter::from(row));
        }

        // Trial ID ごとに Steps をグループ化
        let mut steps_by_trial: std::collections::HashMap<Uuid, Vec<Step>> =
            std::collections::HashMap::new();
        for row in step_rows {
            let trial_id = row.trial_id;
            let params = params_by_step.remove(&row.id).unwrap_or_default();
            steps_by_trial
                .entry(trial_id)
                .or_default()
                .push(row.into_step(params));
        }

        // Trials に Steps を組み立てる
        let trials = trial_rows
            .into_iter()
            .map(|row| {
                let steps = steps_by_trial.remove(&row.id).unwrap_or_default();
                row.into_trial(steps)
            })
            .collect();

        Ok(trials)
    }

    async fn save(&self, trial: &Trial) -> Result<(), RepositoryError> {
        // 1. Trial を UPSERT
        let status_str = trial_status_to_str(trial.status());
        self.executor
            .execute(
                sqlx::query(
                    r#"
                    INSERT INTO trials (id, project_id, name, memo, status, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (id) DO UPDATE SET
                        name = EXCLUDED.name,
                        memo = EXCLUDED.memo,
                        status = EXCLUDED.status,
                        updated_at = EXCLUDED.updated_at
                    "#,
                )
                .bind(trial.id().0)
                .bind(trial.project_id().0)
                .bind(trial.name())
                .bind(trial.memo())
                .bind(status_str)
                .bind(trial.created_at())
                .bind(trial.updated_at()),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        // 2. 既存の Steps を削除（CASCADE で Parameters も削除される）
        self.executor
            .execute(sqlx::query("DELETE FROM steps WHERE trial_id = $1").bind(trial.id().0))
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        // 3. Steps と Parameters を INSERT
        for step in trial.steps() {
            self.executor
                .execute(
                    sqlx::query(
                        r#"
                        INSERT INTO steps (id, trial_id, name, position, started_at, completed_at, created_at, updated_at)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                        "#,
                    )
                    .bind(step.id().0)
                    .bind(step.trial_id().0)
                    .bind(step.name())
                    .bind(step.position())
                    .bind(step.started_at())
                    .bind(step.completed_at())
                    .bind(step.created_at())
                    .bind(step.updated_at()),
                )
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?;

            for param in step.parameters() {
                let content_json = serde_json::to_value(param.content()).map_err(|e| {
                    RepositoryError::Internal {
                        message: format!("Failed to serialize ParameterContent: {e}"),
                    }
                })?;

                self.executor
                    .execute(
                        sqlx::query(
                            r#"
                            INSERT INTO parameters (id, step_id, content, created_at, updated_at)
                            VALUES ($1, $2, $3, $4, $5)
                            "#,
                        )
                        .bind(param.id().0)
                        .bind(param.step_id().0)
                        .bind(content_json)
                        .bind(param.created_at())
                        .bind(param.updated_at()),
                    )
                    .await
                    .map_err(|e| RepositoryError::Internal {
                        message: e.to_string(),
                    })?;
            }
        }

        Ok(())
    }

    async fn delete(&self, id: &TrialId) -> Result<(), RepositoryError> {
        self.executor
            .execute(sqlx::query("DELETE FROM trials WHERE id = $1").bind(id.0))
            .await
            .map(|_| ())
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{DurationUnit, DurationValue, ParameterContent, ParameterValue};
    use crate::ports::{SortDirection, TrialSortColumn};
    use sqlx::PgPool;

    /// テスト用プロジェクトを作成する
    async fn insert_test_project(pool: &PgPool, id: Uuid) {
        sqlx::query(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(id)
        .bind(format!("test-project-{}", &id.to_string()[..8]))
        .execute(pool)
        .await
        .expect("Failed to insert test project");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_and_find_trial(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(
            ProjectId(project_id),
            Some("バゲット第1回".to_string()),
            Some("初回の試行".to_string()),
        );
        let trial_id = trial.id().clone();

        repo.save(&trial).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap().unwrap();
        assert_eq!(found.id(), &trial_id);
        assert_eq!(found.name(), Some("バゲット第1回"));
        assert_eq!(found.memo(), Some("初回の試行"));
        assert!(found.steps().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_trial_with_steps(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), Some("ピザ生地".to_string()), None);

        let step1 = Step::new(trial.id().clone(), "捏ね".to_string(), 0);
        let step2 = Step::new(trial.id().clone(), "一次発酵".to_string(), 1);
        let step3 = Step::new(trial.id().clone(), "焼成".to_string(), 2);

        let trial_with_steps = Trial::from_raw(
            trial.id().clone(),
            trial.project_id().clone(),
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            trial.status().clone(),
            vec![step1, step2, step3],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let trial_id = trial_with_steps.id().clone();

        repo.save(&trial_with_steps).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap().unwrap();
        assert_eq!(found.steps().len(), 3);
        assert_eq!(found.steps()[0].name(), "捏ね");
        assert_eq!(found.steps()[0].position(), 0);
        assert_eq!(found.steps()[1].name(), "一次発酵");
        assert_eq!(found.steps()[1].position(), 1);
        assert_eq!(found.steps()[2].name(), "焼成");
        assert_eq!(found.steps()[2].position(), 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_trial_with_parameters(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), Some("バゲット".to_string()), None);
        let step = Step::new(trial.id().clone(), "捏ね".to_string(), 0);

        let param1 = Parameter::new(
            step.id().clone(),
            ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            },
        );
        let param2 = Parameter::new(
            step.id().clone(),
            ParameterContent::Duration {
                duration: DurationValue::new(15.0, DurationUnit::Minute),
                note: "手捏ね".to_string(),
            },
        );

        let step_with_params = Step::from_raw(
            step.id().clone(),
            step.trial_id().clone(),
            step.name().to_string(),
            step.position(),
            step.started_at().cloned(),
            step.completed_at().cloned(),
            vec![param1, param2],
            *step.created_at(),
            *step.updated_at(),
        );

        let trial_with_data = Trial::from_raw(
            trial.id().clone(),
            trial.project_id().clone(),
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            trial.status().clone(),
            vec![step_with_params],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let trial_id = trial_with_data.id().clone();

        repo.save(&trial_with_data).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap().unwrap();
        assert_eq!(found.steps().len(), 1);
        assert_eq!(found.steps()[0].parameters().len(), 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_project_id(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        let other_project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;
        insert_test_project(&pool, other_project_id).await;

        // 同じプロジェクトに 2 つの Trial を作成
        let trial1 = Trial::new(ProjectId(project_id), Some("試行1".to_string()), None);
        let trial2 = Trial::new(ProjectId(project_id), Some("試行2".to_string()), None);
        // 別プロジェクトに 1 つの Trial
        let trial3 = Trial::new(
            ProjectId(other_project_id),
            Some("別プロジェクト試行".to_string()),
            None,
        );

        repo.save(&trial1).await.unwrap();
        repo.save(&trial2).await.unwrap();
        repo.save(&trial3).await.unwrap();

        let sort = TrialSort::new(TrialSortColumn::CreatedAt, SortDirection::Asc);
        let found = repo
            .find_by_project_id(&ProjectId(project_id), sort)
            .await
            .unwrap();

        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|t| t.project_id().0 == project_id));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_trial(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), Some("更新前".to_string()), None);
        let trial_id = trial.id().clone();
        repo.save(&trial).await.unwrap();

        // 更新
        let updated = Trial::from_raw(
            trial_id.clone(),
            ProjectId(project_id),
            Some("更新後".to_string()),
            Some("メモ追加".to_string()),
            trial.status().clone(),
            Vec::new(),
            *trial.created_at(),
            chrono::Utc::now(),
        );
        repo.save(&updated).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap().unwrap();
        assert_eq!(found.name(), Some("更新後"));
        assert_eq!(found.memo(), Some("メモ追加"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_trial(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), None, None);
        let trial_id = trial.id().clone();
        repo.save(&trial).await.unwrap();

        repo.delete(&trial_id).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_cascades_to_steps(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), None, None);
        let step = Step::new(trial.id().clone(), "テスト工程".to_string(), 0);
        let step_id = step.id().0;

        let trial_with_step = Trial::from_raw(
            trial.id().clone(),
            trial.project_id().clone(),
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            trial.status().clone(),
            vec![step],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let trial_id = trial_with_step.id().clone();

        repo.save(&trial_with_step).await.unwrap();

        // Trial を削除
        repo.delete(&trial_id).await.unwrap();

        // Steps もカスケード削除されている
        let step_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM steps WHERE id = $1")
            .bind(step_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(step_count.0, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_parameter_content_json_roundtrip(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));
        let project_id = Uuid::new_v4();
        insert_test_project(&pool, project_id).await;

        let trial = Trial::new(ProjectId(project_id), None, None);
        let step = Step::new(trial.id().clone(), "テスト".to_string(), 0);

        let contents = vec![
            ParameterContent::KeyValue {
                key: "水温".to_string(),
                value: ParameterValue::Quantity {
                    amount: 28.0,
                    unit: "C".to_string(),
                },
            },
            ParameterContent::KeyValue {
                key: "発酵場所".to_string(),
                value: ParameterValue::Text {
                    value: "冷蔵庫".to_string(),
                },
            },
            ParameterContent::Duration {
                duration: DurationValue::new(90.0, DurationUnit::Minute),
                note: "一次発酵".to_string(),
            },
            ParameterContent::TimeMarker {
                at: DurationValue::new(30.0, DurationUnit::Minute),
                note: "温度を220度に下げる".to_string(),
            },
            ParameterContent::Text {
                value: "丸め成型".to_string(),
            },
        ];

        let params: Vec<Parameter> = contents
            .iter()
            .map(|c| Parameter::new(step.id().clone(), c.clone()))
            .collect();

        let step_with_params = Step::from_raw(
            step.id().clone(),
            step.trial_id().clone(),
            step.name().to_string(),
            step.position(),
            step.started_at().cloned(),
            step.completed_at().cloned(),
            params,
            *step.created_at(),
            *step.updated_at(),
        );

        let trial_with_data = Trial::from_raw(
            trial.id().clone(),
            trial.project_id().clone(),
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            trial.status().clone(),
            vec![step_with_params],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let trial_id = trial_with_data.id().clone();

        repo.save(&trial_with_data).await.unwrap();

        let found = repo.find_by_id(&trial_id).await.unwrap().unwrap();
        let found_params = found.steps()[0].parameters();
        assert_eq!(found_params.len(), 5);

        // 各パラメーターの内容を検証
        for (original, found_param) in contents.iter().zip(found_params.iter()) {
            assert_eq!(original, found_param.content());
        }
    }
}
