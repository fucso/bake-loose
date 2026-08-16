//! PgTrialRepository 実装

use std::collections::HashMap;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::parameter::Parameter;
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;
use crate::ports::trial_repository::TrialRepository;

use super::executor::PgExecutor;
use super::models::{ParameterRow, StepRow, TrialRow};

/// PostgreSQL 用の TrialRepository 実装
///
/// `PgExecutor` を使用して、pool 直接または
/// トランザクション内のどちらでも動作する。
#[derive(Clone)]
pub struct PgTrialRepository {
    executor: PgExecutor,
}

impl PgTrialRepository {
    /// 新しい PgTrialRepository を作成する
    pub fn new(executor: PgExecutor) -> Self {
        Self { executor }
    }

    /// 指定した Trial ID 群に紐づく Step を Parameter込みで取得する
    ///
    /// Step・Parameter をそれぞれ一括取得してから Rust 側で組み立てることで、
    /// Trial 件数・Step 件数に対する N+1 クエリを避ける。
    async fn fetch_steps_by_trial_ids(
        &self,
        trial_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<Step>>, RepositoryError> {
        if trial_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let step_rows = self
            .executor
            .fetch_all(
                sqlx::query_as::<_, StepRow>(
                    "SELECT * FROM steps WHERE trial_id = ANY($1) ORDER BY trial_id, position",
                )
                .bind(trial_ids),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        let step_ids: Vec<Uuid> = step_rows.iter().map(|row| row.id).collect();

        let mut parameters_by_step: HashMap<Uuid, Vec<Parameter>> = HashMap::new();
        if !step_ids.is_empty() {
            let parameter_rows = self
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

            for row in parameter_rows {
                parameters_by_step
                    .entry(row.step_id)
                    .or_default()
                    .push(row.into());
            }
        }

        let mut steps_by_trial: HashMap<Uuid, Vec<Step>> = HashMap::new();
        for row in step_rows {
            let trial_id = row.trial_id;
            let parameters = parameters_by_step.remove(&row.id).unwrap_or_default();
            steps_by_trial
                .entry(trial_id)
                .or_default()
                .push(row.into_domain(parameters));
        }

        Ok(steps_by_trial)
    }
}

#[async_trait]
impl TrialRepository for PgTrialRepository {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        let query = sqlx::query_as::<_, TrialRow>("SELECT * FROM trials WHERE id = $1").bind(id.0);

        let trial_row =
            self.executor
                .fetch_optional(query)
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?;

        let Some(trial_row) = trial_row else {
            return Ok(None);
        };

        let mut steps_by_trial = self.fetch_steps_by_trial_ids(&[trial_row.id]).await?;
        let steps = steps_by_trial.remove(&trial_row.id).unwrap_or_default();

        Ok(Some(trial_row.into_domain(steps)))
    }

    async fn find_all_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Trial>, RepositoryError> {
        // created_at が同時刻になり得るため id もキーに加えて順序を決定的にする
        let query = sqlx::query_as::<_, TrialRow>(
            "SELECT * FROM trials WHERE project_id = $1 ORDER BY created_at, id",
        )
        .bind(project_id.0);

        let trial_rows =
            self.executor
                .fetch_all(query)
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?;

        let trial_ids: Vec<Uuid> = trial_rows.iter().map(|row| row.id).collect();
        let mut steps_by_trial = self.fetch_steps_by_trial_ids(&trial_ids).await?;

        Ok(trial_rows
            .into_iter()
            .map(|row| {
                let steps = steps_by_trial.remove(&row.id).unwrap_or_default();
                row.into_domain(steps)
            })
            .collect())
    }

    async fn save(&self, trial: &Trial) -> Result<(), RepositoryError> {
        let status = TrialRow::status_column(trial.status());

        self.executor
            .execute(
                sqlx::query(
                    r#"
                    INSERT INTO trials (id, project_id, name, memo, status, completed_at, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                    ON CONFLICT (id) DO UPDATE SET
                        name = EXCLUDED.name,
                        memo = EXCLUDED.memo,
                        status = EXCLUDED.status,
                        completed_at = EXCLUDED.completed_at,
                        updated_at = NOW()
                    "#,
                )
                .bind(trial.id().0)
                .bind(trial.project_id().0)
                .bind(trial.name())
                .bind(trial.memo())
                .bind(status)
                .bind(trial.completed_at().copied().map(|d| d.into_fixed_offset())),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        // aggregate から取り除かれた Step を削除する（cascade で Parameter も削除される）
        let step_ids: Vec<Uuid> = trial.steps().iter().map(|step| step.id().0).collect();

        self.executor
            .execute(
                sqlx::query("DELETE FROM steps WHERE trial_id = $1 AND NOT (id = ANY($2))")
                    .bind(trial.id().0)
                    .bind(step_ids),
            )
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        for step in trial.steps() {
            self.executor
                .execute(
                    sqlx::query(
                        r#"
                        INSERT INTO steps (id, trial_id, name, position, started_at, completed_at, created_at, updated_at)
                        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                        ON CONFLICT (id) DO UPDATE SET
                            name = EXCLUDED.name,
                            position = EXCLUDED.position,
                            started_at = EXCLUDED.started_at,
                            completed_at = EXCLUDED.completed_at,
                            updated_at = NOW()
                        "#,
                    )
                    .bind(step.id().0)
                    .bind(trial.id().0)
                    .bind(step.name())
                    .bind(step.position())
                    .bind(step.started_at().copied().map(|d| d.into_fixed_offset()))
                    .bind(step.completed_at().copied().map(|d| d.into_fixed_offset())),
                )
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?;

            // aggregate から取り除かれた Parameter を削除する
            let parameter_ids: Vec<Uuid> =
                step.parameters().iter().map(|param| param.id().0).collect();

            self.executor
                .execute(
                    sqlx::query("DELETE FROM parameters WHERE step_id = $1 AND NOT (id = ANY($2))")
                        .bind(step.id().0)
                        .bind(parameter_ids),
                )
                .await
                .map_err(|e| RepositoryError::Internal {
                    message: e.to_string(),
                })?;

            for parameter in step.parameters() {
                self.executor
                    .execute(
                        sqlx::query(
                            r#"
                            INSERT INTO parameters (id, step_id, content, created_at, updated_at)
                            VALUES ($1, $2, $3, NOW(), NOW())
                            ON CONFLICT (id) DO UPDATE SET
                                content = EXCLUDED.content,
                                updated_at = NOW()
                            "#,
                        )
                        .bind(parameter.id().0)
                        .bind(step.id().0)
                        .bind(sqlx::types::Json(parameter.content().clone())),
                    )
                    .await
                    .map_err(|e| RepositoryError::Internal {
                        message: e.to_string(),
                    })?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::ParameterContent;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::TrialStatus;
    use sqlx::PgPool;

    /// テスト用の Project 行を投入する（Trial の外部キー制約を満たすため）
    ///
    /// name は一意制約があるため、id を含めて衝突を避ける
    async fn insert_test_project(pool: &PgPool, id: Uuid) {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(format!("テスト用プロジェクト-{id}"))
        .execute(pool)
        .await
        .expect("Failed to insert test project");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_returns_none_when_not_exists(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool));

        let result = repo.find_by_id(&TrialId::new()).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_and_find_by_id_roundtrip(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));

        let project_id = ProjectId::new();
        insert_test_project(&pool, project_id.0).await;

        let mut trial = Trial::new(
            project_id.clone(),
            Some("焼成温度検証".to_string()),
            Some("メモ".to_string()),
        );
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        step.add_parameter(Parameter::new(
            step.id().clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        ));
        trial.add_step(step);

        let result = repo.save(&trial).await;
        assert!(result.is_ok());

        let found = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert_eq!(found.id(), trial.id());
        assert_eq!(found.project_id(), &project_id);
        assert_eq!(found.name(), Some("焼成温度検証"));
        assert_eq!(found.memo(), Some("メモ"));
        assert_eq!(found.status(), &TrialStatus::InProgress);
        assert_eq!(found.steps().len(), 1);
        assert_eq!(found.steps()[0].name(), "こね");
        assert_eq!(found.steps()[0].parameters().len(), 1);
        assert_eq!(
            found.steps()[0].parameters()[0].content(),
            &ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            }
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_updates_existing_trial(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));

        let project_id = ProjectId::new();
        insert_test_project(&pool, project_id.0).await;

        let mut trial = Trial::new(project_id, Some("更新前".to_string()), None);
        repo.save(&trial).await.unwrap();

        trial.set_name(Some("更新後".to_string()));
        trial.complete(None);
        repo.save(&trial).await.unwrap();

        let found = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert_eq!(found.name(), Some("更新後"));
        assert_eq!(found.status(), &TrialStatus::Completed);
        // DB は マイクロ秒精度のため、ナノ秒まで含む厳密一致ではなくマイクロ秒単位で比較する
        assert_eq!(
            found
                .completed_at()
                .map(|d| d.into_fixed_offset().timestamp_micros()),
            trial
                .completed_at()
                .map(|d| d.into_fixed_offset().timestamp_micros())
        );
        assert!(found.completed_at().is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_all_by_project_returns_all_trials_for_project(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));

        let project_id = ProjectId::new();
        insert_test_project(&pool, project_id.0).await;
        let other_project_id = ProjectId::new();
        insert_test_project(&pool, other_project_id.0).await;

        let trial1 = Trial::new(project_id.clone(), Some("試行1".to_string()), None);
        let trial2 = Trial::new(project_id.clone(), Some("試行2".to_string()), None);
        let other_trial = Trial::new(other_project_id, Some("別プロジェクト".to_string()), None);

        repo.save(&trial1).await.unwrap();
        repo.save(&trial2).await.unwrap();
        repo.save(&other_trial).await.unwrap();

        let result = repo.find_all_by_project(&project_id).await;
        assert!(result.is_ok());
        let trials = result.unwrap();
        assert_eq!(trials.len(), 2);
        assert!(trials.iter().all(|t| t.project_id() == &project_id));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_removes_steps_not_in_aggregate(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));

        let project_id = ProjectId::new();
        insert_test_project(&pool, project_id.0).await;

        let mut trial = Trial::new(project_id, None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        repo.save(&trial).await.unwrap();

        let found = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert_eq!(found.steps().len(), 1);

        // aggregate から Step を取り除いて保存し直す
        trial.steps_mut().retain(|s| s.id() != &step_id);
        repo.save(&trial).await.unwrap();

        let found_after = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert!(found_after.steps().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_removes_parameters_not_in_aggregate(pool: PgPool) {
        let repo = PgTrialRepository::new(PgExecutor::from_pool(pool.clone()));

        let project_id = ProjectId::new();
        insert_test_project(&pool, project_id.0).await;

        let mut trial = Trial::new(project_id, None, None);
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let parameter = Parameter::new(
            step.id().clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        let step_id = step.id().clone();
        trial.add_step(step);
        repo.save(&trial).await.unwrap();

        let found = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert_eq!(found.steps()[0].parameters().len(), 1);

        // aggregate から Parameter を取り除いて保存し直す
        let step_mut = trial
            .steps_mut()
            .iter_mut()
            .find(|s| s.id() == &step_id)
            .unwrap();
        step_mut.remove_parameter(&parameter_id);
        repo.save(&trial).await.unwrap();

        let found_after = repo.find_by_id(trial.id()).await.unwrap().unwrap();
        assert!(found_after.steps()[0].parameters().is_empty());
    }
}
