//! complete_trial ユースケース
//!
//! trial_id で Trial を取得し complete_trial ドメインアクションを適用・保存する。

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::complete_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

/// ユースケースの入力
///
/// completed_at が未指定の場合は現在時刻が採用される。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub trial_id: Uuid,
    pub completed_at: Option<DateTime<FixedOffset>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(complete_trial::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial を取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow
        .trial_repository()
        .find_by_id(&trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        Some(trial) => trial,
        None => return Err(Error::NotFound),
    };

    // 2. ドメインアクション実行
    let command = complete_trial::Command {
        completed_at: input.completed_at.map(JstDateTime::from_fixed_offset),
    };
    let completed = match complete_trial::run(trial, command) {
        Ok(trial) => trial,
        Err(e) => return Err(Error::Domain(e)),
    };

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&completed).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

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
        uow.trial_repository().save(&trial).await.unwrap();

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
            .find_by_id(&trial_id)
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
        uow.trial_repository().save(&trial).await.unwrap();

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
        uow.trial_repository().save(&trial).await.unwrap();

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
}
