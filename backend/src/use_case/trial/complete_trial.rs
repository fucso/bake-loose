//! complete_trial ユースケース

use crate::domain::actions::trial::complete_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub trial_id: TrialId,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(complete_trial::Error),
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
    let trial = match complete_trial::run(trial) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::trial::complete_trial;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_complete_trial_success() {
        let mut uow = MockUnitOfWork::default();

        // InProgress な Trial を作成して保存
        let trial = Trial::new(ProjectId::new(), Some("テスト試行".to_string()), None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input { trial_id };
        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let completed = result.unwrap();
        assert_eq!(completed.status(), &TrialStatus::Completed);
    }

    #[tokio::test]
    async fn test_returns_error_when_trial_not_found() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: TrialId::new(),
        };
        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::TrialNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_already_completed() {
        let mut uow = MockUnitOfWork::default();

        // 完了済みの Trial を作成して保存
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id.clone(), None, None);
        let trial_id = trial.id().clone();
        let completed_trial = Trial::from_raw(
            trial_id.clone(),
            project_id,
            None,
            None,
            TrialStatus::Completed,
            vec![],
            *trial.created_at(),
            *trial.updated_at(),
        );
        uow.trial_repository().save(&completed_trial).await.unwrap();

        let input = Input { trial_id };
        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_trial::Error::AlreadyCompleted)
        );
    }
}
