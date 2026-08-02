//! complete_step ユースケース

use chrono::{DateTime, FixedOffset};

use crate::domain::actions::trial::complete_step;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub trial_id: TrialId,
    pub step_id: StepId,
    pub completed_at: Option<DateTime<FixedOffset>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(complete_step::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Trial取得
    let trial = uow
        .trial_repository()
        .find_by_id(&input.trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    let trial = match trial {
        Some(trial) => trial,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::NotFound);
        }
    };

    // 3. ドメインアクション実行
    let command = complete_step::Command {
        step_id: input.step_id,
        completed_at: input.completed_at,
    };
    let trial = match complete_step::run(trial, command) {
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
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::use_case::test::MockUnitOfWork;

    async fn seed_trial_with_step(uow: &mut MockUnitOfWork) -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        uow.trial_repository().save(&trial).await.unwrap();
        (trial, step_id)
    }

    #[tokio::test]
    async fn test_execute_completes_step_successfully() {
        let mut uow = MockUnitOfWork::default();
        let (trial, step_id) = seed_trial_with_step(&mut uow).await;

        let input = Input {
            trial_id: trial.id().clone(),
            step_id: step_id.clone(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated_trial = result.unwrap();
        let step = updated_trial
            .steps()
            .iter()
            .find(|s| s.id() == &step_id)
            .unwrap();
        assert!(step.is_completed());

        // 永続化されていることを確認
        let saved_trial = uow
            .trial_repository()
            .find_by_id(trial.id())
            .await
            .unwrap()
            .unwrap();
        let saved_step = saved_trial
            .steps()
            .iter()
            .find(|s| s.id() == &step_id)
            .unwrap();
        assert!(saved_step.is_completed());
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            trial_id: TrialId::new(),
            step_id: StepId::new(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_step_not_found() {
        let mut uow = MockUnitOfWork::default();
        let (trial, _) = seed_trial_with_step(&mut uow).await;

        let input = Input {
            trial_id: trial.id().clone(),
            step_id: StepId::new(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::StepNotFound)
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_step_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let (trial, step_id) = seed_trial_with_step(&mut uow).await;

        let input = Input {
            trial_id: trial.id().clone(),
            step_id: step_id.clone(),
            completed_at: None,
        };
        execute(&mut uow, input).await.unwrap();

        let input = Input {
            trial_id: trial.id().clone(),
            step_id,
            completed_at: None,
        };
        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::StepAlreadyCompleted)
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let (mut trial, step_id) = seed_trial_with_step(&mut uow).await;
        trial.complete();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial.id().clone(),
            step_id,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::TrialAlreadyCompleted)
        );
    }
}
