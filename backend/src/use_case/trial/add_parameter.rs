//! add_parameter ユースケース
//!
//! trial_id で Trial を取得し、Step にパラメーターを追加する
//! add_parameter ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::add_parameter;
use crate::domain::models::parameter::ParameterContent;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(add_parameter::Error),
    Infrastructure(String),
}

/// ユースケースの入力
///
/// presentation 層は domain 型を組み立てず、フラットな値のみを渡す。
/// ParameterContent は元々オブジェクト形式の値であるため、無理にフラット化しない。
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub content: ParameterContent,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    let trial_id = TrialId(input.trial_id);
    let trial = match uow.trial_repository().find_by_id(&trial_id).await {
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

    let trial = match add_parameter::run(
        trial,
        add_parameter::Command {
            step_id: StepId(input.step_id),
            content: input.content,
        },
    ) {
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

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[tokio::test]
    async fn test_execute_adds_parameter_to_existing_step() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
        assert_eq!(
            step.parameters()[0].content(),
            &ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            }
        );

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        let saved_step = saved.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(saved_step.parameters().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: Uuid::new_v4(),
            step_id: Uuid::new_v4(),
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::NotFound));
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_step_not_found() {
        let (trial, _step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            step_id: Uuid::new_v4(),
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(add_parameter::Error::StepNotFound))
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_invalid_parameter() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            content: ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: crate::domain::models::parameter::ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "".to_string(),
                },
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(add_parameter::Error::InvalidParameter(
                add_parameter::ParameterValidationError::EmptyQuantityUnit
            )))
        );
    }
}
