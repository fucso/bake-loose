//! update_parameter ユースケース
//!
//! trial_id で Trial を取得し update_parameter ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::update_parameter;
use crate::domain::models::parameter::{ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

/// ユースケースの入力
///
/// ParameterContent は元々オブジェクト形式の値であるため、無理にフラット化しない。
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub parameter_id: Uuid,
    pub content: ParameterContent,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_parameter::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. Trial を取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow
        .trial_repository()
        .find_by_id(&trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        Some(trial) => trial,
        None => {
            let _ = uow.rollback().await;
            return Err(Error::NotFound);
        }
    };

    // 3. ドメインアクション実行
    let command = update_parameter::Command {
        step_id: StepId(input.step_id),
        parameter_id: ParameterId(input.parameter_id),
        content: input.content,
    };
    let updated = match update_parameter::run(trial, command) {
        Ok(trial) => trial,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.trial_repository().save(&updated).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterId};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::{Step, StepId};
    use crate::use_case::test::MockUnitOfWork;

    async fn seed_trial_with_parameter(
        uow: &mut MockUnitOfWork,
        content: ParameterContent,
    ) -> (Trial, StepId, ParameterId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        let parameter = Parameter::new(step_id.clone(), content);
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        trial.add_step(step);
        uow.trial_repository().save(&trial).await.unwrap();
        (trial, step_id, parameter_id)
    }

    #[tokio::test]
    async fn test_execute_updates_parameter_content_successfully() {
        let mut uow = MockUnitOfWork::default();
        let (trial, step_id, parameter_id) = seed_trial_with_parameter(
            &mut uow,
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        )
        .await;

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
            parameter_id: parameter_id.0,
            content: ParameterContent::Text {
                value: "打ち粉を多めに".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(
            step.parameters()[0].content(),
            &ParameterContent::Text {
                value: "打ち粉を多めに".to_string(),
            }
        );

        let saved = uow
            .trial_repository()
            .find_by_id(trial.id())
            .await
            .unwrap()
            .unwrap();
        let saved_step = saved.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(
            saved_step.parameters()[0].content(),
            &ParameterContent::Text {
                value: "打ち粉を多めに".to_string(),
            }
        );
        assert_eq!(saved_step.parameters()[0].id(), &parameter_id);
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            trial_id: Uuid::new_v4(),
            step_id: Uuid::new_v4(),
            parameter_id: Uuid::new_v4(),
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_parameter_not_found() {
        let mut uow = MockUnitOfWork::default();
        let (trial, step_id, _) = seed_trial_with_parameter(
            &mut uow,
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        )
        .await;

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
            parameter_id: Uuid::new_v4(),
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(update_parameter::Error::ParameterNotFound)
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let (mut trial, step_id, parameter_id) = seed_trial_with_parameter(
            &mut uow,
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        )
        .await;
        trial.complete(None);
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
            parameter_id: parameter_id.0,
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(update_parameter::Error::TrialAlreadyCompleted)
        );
    }
}
