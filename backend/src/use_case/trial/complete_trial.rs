//! complete_trial ユースケース
//!
//! trial_id で Trial を取得し complete_trial ドメインアクションを適用・保存する。

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::complete_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::{TrialRepository, TrialScope};
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

/// completed_at が未指定の場合は現在時刻が採用される。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub trial_id: Uuid,
    pub completed_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(complete_trial::Error),
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

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial を取得
    // status/completed_at のみを変更するため Step/Parameter は不要（TrialOnly）
    let trial_id = TrialId(input.trial_id);
    let trial = match uow
        .trial_repository()
        .find_by_id(&trial_id, TrialScope::TrialOnly)
        .await
    {
        Ok(Some(trial)) => trial,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    };

    // 2. ドメインアクション実行
    let command = complete_trial::Command {
        completed_at: input.completed_at.map(JstDateTime::from_fixed_offset),
    };
    let completed = complete_trial::run(trial, command).map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_trial(uow, &completed, TrialScope::TrialOnly).await?;

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;
    use crate::use_case::test::MockUnitOfWork;

    fn in_progress_trial() -> Trial {
        Trial::new(ProjectId::new(), None, None)
    }

    #[tokio::test]
    async fn test_execute_completes_trial_successfully() {
        let mut uow = MockUnitOfWork::default();
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

        let input = Input {
            trial_id: trial_id.0,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let completed = result.unwrap();
        assert_eq!(completed.status(), &TrialStatus::Completed);
        assert!(completed.completed_at().is_some());

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id, TrialScope::Full)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.status(), &TrialStatus::Completed);
    }

    #[tokio::test]
    async fn test_execute_uses_specified_completed_at() {
        let mut uow = MockUnitOfWork::default();
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

        let completed_at = JstDateTime::now().into_fixed_offset();
        let input = Input {
            trial_id: trial_id.0,
            completed_at: Some(completed_at),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let completed = result.unwrap();
        assert_eq!(
            completed.completed_at(),
            Some(&JstDateTime::from_fixed_offset(completed_at))
        );
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            trial_id: Uuid::new_v4(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let mut trial = in_progress_trial();
        trial.complete(None);
        let trial_id = trial.id().clone();
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

        let input = Input {
            trial_id: trial_id.0,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_trial::Error::TrialAlreadyCompleted)
        );
    }

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            trial_id: trial_id.0,
            completed_at: None,
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
