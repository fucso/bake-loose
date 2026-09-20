//! update_step ユースケース
//!
//! trial_id で Trial を取得し、Step の name/started_at 更新（update_step アクション）を
//! 適用・保存する。パラメーターの追加・削除はそれぞれ add_parameter / remove_parameter
//! ユースケースが担当する。

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::update_step;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_step::Error),
    ReferenceNotFound { entity: String },
    Conflict { entity: String, field: String },
    Infrastructure(String),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Conflict { entity, field } => Error::Conflict { entity, field },
            // 参照先が Trial とは限らない（例: parameters_step_id_fkey なら Step）ため、
            // エンティティ名を捨てずに presentation 層へ引き渡す
            RepositoryError::NotFound { entity, .. } => Error::ReferenceNotFound { entity },
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    /// Some の場合のみ変更
    pub name: Option<String>,
    /// None: 変更なし / Some(None): クリア / Some(Some(t)): t に設定
    pub started_at: Option<Option<DateTime<FixedOffset>>>,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow.trial_repository().find_by_id(&trial_id).await {
        Ok(Some(trial)) => trial,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    };

    // 2. ドメインアクション実行
    let trial = update_step::run(
        trial,
        update_step::Command {
            step_id: StepId(input.step_id),
            name: input.name,
            started_at: input
                .started_at
                .map(|opt| opt.map(JstDateTime::from_fixed_offset)),
        },
    )
    .map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_trial(uow, &trial).await?;

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::use_case::test::MockUnitOfWork;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    fn base_input(trial_id: Uuid, step_id: Uuid) -> Input {
        Input {
            trial_id,
            step_id,
            name: None,
            started_at: None,
        }
    }

    #[tokio::test]
    async fn test_update_step_name_success() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.name(), "新名称");

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        let saved_step = saved.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(saved_step.name(), "新名称");
    }

    #[tokio::test]
    async fn test_update_step_sets_and_clears_started_at() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let started_at = DateTime::parse_from_rfc3339("2026-01-01T09:00:00+09:00").unwrap();
        let input = Input {
            started_at: Some(Some(started_at)),
            ..base_input(trial_id.0, step_id.0)
        };
        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.started_at().is_some());

        let input = Input {
            started_at: Some(None),
            ..base_input(updated.id().0, step_id.0)
        };
        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.started_at().is_none());
    }

    #[tokio::test]
    async fn test_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();

        let input = base_input(Uuid::new_v4(), Uuid::new_v4());

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::NotFound));
    }

    #[tokio::test]
    async fn test_propagates_domain_error_when_step_not_found() {
        let (trial, _step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, Uuid::new_v4())
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::Domain(update_step::Error::StepNotFound)));

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved, trial);
    }

    #[tokio::test]
    async fn test_propagates_domain_error_when_trial_completed() {
        let (mut trial, step_id) = trial_with_step();
        trial.complete(None);
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(update_step::Error::TrialAlreadyCompleted))
        );
    }

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert!(matches!(result, Err(Error::Infrastructure(_))));
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }
}
