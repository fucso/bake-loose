//! add_step ユースケース
//!
//! trial_id で Trial を取得し、add_step ドメインアクションを適用・保存する。

use crate::domain::actions::trial::add_step;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

/// ユースケースの入力
pub struct Input {
    pub trial_id: TrialId,
    pub command: add_step::Command,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(add_step::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Trial取得
    let trial = match uow.trial_repository().find_by_id(&input.trial_id).await {
        Ok(Some(trial)) => trial,
        Ok(None) => {
            let _ = uow.rollback().await;
            return Err(Error::NotFound);
        }
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Infrastructure(format!("{:?}", e)));
        }
    };

    // 3. ドメインアクション実行
    let trial = match add_step::run(trial, input.command) {
        Ok(trial) => trial,
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
    use crate::domain::actions::trial::add_step;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::Trial;
    use crate::use_case::test::MockUnitOfWork;

    fn command(name: &str) -> add_step::Command {
        add_step::Command {
            name: name.to_string(),
            started_at: None,
            parameters: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_execute_adds_step_to_existing_trial() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.clone(),
            command: command("こね"),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.steps().len(), 1);
        assert_eq!(updated.steps()[0].name(), "こね");

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.steps().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let non_existing_id = TrialId::new();

        let input = Input {
            trial_id: non_existing_id,
            command: command("こね"),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete();
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id,
            command: command("こね"),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(add_step::Error::TrialAlreadyCompleted)
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_invalid_step_name() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id,
            command: command(""),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(add_step::Error::InvalidStepName(
                add_step::StepNameError::EmptyName
            ))
        );
    }
}
