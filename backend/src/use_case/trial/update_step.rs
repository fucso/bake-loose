//! update_step ユースケース
//!
//! trial_id で Trial を取得し update_step ドメインアクションを適用・保存する。

use crate::domain::actions::trial::update_step;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_step::Error),
    Infrastructure(String),
}

pub struct Input {
    pub trial_id: TrialId,
    pub command: update_step::Command,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

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

    let trial = match update_step::run(trial, input.command) {
        Ok(trial) => trial,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    if let Err(e) = uow.trial_repository().save(&trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

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

    fn trial_with_step() -> (Trial, crate::domain::models::step::StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[tokio::test]
    async fn test_update_step_name_success() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.clone(),
            command: update_step::Command {
                step_id: step_id.clone(),
                name: Some("新名称".to_string()),
                started_at: None,
                add_parameters: Vec::new(),
                remove_parameter_ids: Vec::new(),
            },
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
    async fn test_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: TrialId::new(),
            command: update_step::Command {
                step_id: crate::domain::models::step::StepId::new(),
                name: Some("新名称".to_string()),
                started_at: None,
                add_parameters: Vec::new(),
                remove_parameter_ids: Vec::new(),
            },
        };

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
            trial_id: trial_id.clone(),
            command: update_step::Command {
                step_id: crate::domain::models::step::StepId::new(),
                name: Some("新名称".to_string()),
                started_at: None,
                add_parameters: Vec::new(),
                remove_parameter_ids: Vec::new(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::Domain(update_step::Error::StepNotFound)));

        // ドメインエラー時は永続化されていないこと
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
        trial.complete();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.clone(),
            command: update_step::Command {
                step_id,
                name: Some("新名称".to_string()),
                started_at: None,
                add_parameters: Vec::new(),
                remove_parameter_ids: Vec::new(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(update_step::Error::TrialAlreadyCompleted))
        );
    }
}
