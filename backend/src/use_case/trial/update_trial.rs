//! update_trial ユースケース

use crate::domain::actions::trial::update_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
#[derive(Debug)]
pub struct Input {
    pub trial_id: TrialId,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(update_trial::Error),
    TrialNotFound,
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Trial を取得
    let trial = match uow
        .trial_repository()
        .find_by_id(&input.trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        Some(t) => t,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::TrialNotFound);
        }
    };

    // 3. ドメインアクション実行
    let command = update_trial::Command {
        name: input.name,
        memo: input.memo,
    };
    let updated_trial = match update_trial::run(trial, command) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&updated_trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(updated_trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::trial::update_trial as domain_update_trial;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::{TrialId, TrialStatus};
    use crate::use_case::test::MockUnitOfWork;

    fn make_in_progress_trial() -> Trial {
        Trial::new(
            ProjectId::new(),
            Some("テスト試行".to_string()),
            Some("メモ".to_string()),
        )
    }

    fn make_completed_trial() -> Trial {
        Trial::from_raw(
            TrialId::new(),
            ProjectId::new(),
            Some("完了済み試行".to_string()),
            None,
            TrialStatus::Completed,
            vec![],
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_update_trial_name() {
        let trial = make_in_progress_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            name: Some("新しい名前".to_string()),
            memo: None,
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name(), Some("新しい名前"));
        // memo は既存値が維持される
        assert_eq!(updated.memo(), Some("メモ"));
    }

    #[tokio::test]
    async fn test_update_trial_memo() {
        let trial = make_in_progress_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            name: None,
            memo: Some("新しいメモ".to_string()),
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        // name は既存値が維持される
        assert_eq!(updated.name(), Some("テスト試行"));
        assert_eq!(updated.memo(), Some("新しいメモ"));
    }

    #[tokio::test]
    async fn test_returns_error_when_trial_not_found() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: TrialId::new(),
            name: Some("名前".to_string()),
            memo: None,
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::TrialNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_completed() {
        let trial = make_completed_trial();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            name: Some("更新".to_string()),
            memo: None,
        };

        let result = execute(&mut uow, input).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(domain_update_trial::Error::TrialAlreadyCompleted)
        );
    }
}
