//! complete_step ユースケース

use chrono::{DateTime, Utc};

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
    pub completed_at: Option<DateTime<Utc>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(complete_step::Error),
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
    let trial = uow
        .trial_repository()
        .find_by_id(&input.trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    let trial = match trial {
        Some(t) => t,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::TrialNotFound);
        }
    };

    // 3. ドメインアクション実行
    let command = complete_step::Command {
        step_id: input.step_id,
        completed_at: input.completed_at,
    };
    let trial = match complete_step::run(trial, command) {
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
    use crate::domain::actions::trial::complete_step;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::{Step, StepId};
    use crate::domain::models::trial::{Trial, TrialId, TrialStatus};
    use crate::use_case::test::MockUnitOfWork;

    fn make_trial_with_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let past = Utc::now() - chrono::Duration::seconds(10);

        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "捏ね".to_string(),
            0,
            None,
            None,
            vec![],
            past,
            past,
        );

        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::InProgress,
            vec![step],
            past,
            past,
        );

        (trial, step_id)
    }

    fn make_trial_with_completed_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let past = Utc::now() - chrono::Duration::seconds(10);
        let completed_past = Utc::now() - chrono::Duration::seconds(5);

        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "捏ね".to_string(),
            0,
            None,
            Some(completed_past),
            vec![],
            past,
            past,
        );

        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::InProgress,
            vec![step],
            past,
            past,
        );

        (trial, step_id)
    }

    #[tokio::test]
    async fn test_complete_step_success() {
        let (trial, step_id) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: step_id.clone(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.completed_at().is_some());
    }

    #[tokio::test]
    async fn test_complete_step_with_specified_time() {
        let (trial, step_id) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let specified_time = Utc::now() - chrono::Duration::hours(1);
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: step_id.clone(),
            completed_at: Some(specified_time),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.completed_at(), Some(&specified_time));
    }

    #[tokio::test]
    async fn test_returns_error_when_trial_not_found() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: TrialId::new(),
            step_id: StepId::new(),
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::TrialNotFound);
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_step_not_found() {
        let (trial, _) = make_trial_with_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id: StepId::new(), // non-existent step
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::StepNotFound)
        );
    }

    #[tokio::test]
    async fn test_returns_domain_error_when_already_completed() {
        let (trial, step_id) = make_trial_with_completed_step();
        let trial_id = trial.id().clone();
        let mut uow = MockUnitOfWork::with_trials(vec![trial]);

        let input = Input {
            trial_id,
            step_id,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::StepAlreadyCompleted)
        );
    }
}
