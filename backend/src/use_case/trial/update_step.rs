//! update_step ユースケース
//!
//! trial_id で Trial を取得し、Step の name/started_at 更新（update_step アクション）・
//! パラメーターの追加（add_parameter アクション）・削除（remove_parameter アクション）を
//! それぞれ独立したドメインアクションとして順に適用し、保存する。

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::{add_parameter, remove_parameter, update_step};
use crate::domain::models::parameter::{ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_step::Error),
    AddParameterDomain {
        parameter_index: usize,
        source: add_parameter::Error,
    },
    RemoveParameterDomain(remove_parameter::Error),
    Infrastructure(String),
}

/// ユースケースの入力
///
/// presentation 層は domain 型を組み立てず、フラットな値のみを渡す。
/// ParameterContent は元々オブジェクト形式の値であるため、無理にフラット化しない。
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    /// Some の場合のみ変更
    pub name: Option<String>,
    /// None: 変更なし / Some(None): クリア / Some(Some(t)): t に設定
    pub started_at: Option<Option<DateTime<FixedOffset>>>,
    pub add_parameters: Vec<ParameterContent>,
    pub remove_parameter_ids: Vec<Uuid>,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    let trial_id = TrialId(input.trial_id);
    let step_id = StepId(input.step_id);

    let mut trial = match uow.trial_repository().find_by_id(&trial_id).await {
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

    // 1. name/started_at の更新
    trial = match update_step::run(
        trial,
        update_step::Command {
            step_id: step_id.clone(),
            name: input.name,
            started_at: input
                .started_at
                .map(|opt| opt.map(JstDateTime::from_fixed_offset)),
        },
    ) {
        Ok(trial) => trial,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 2. パラメーターの追加
    for (parameter_index, content) in input.add_parameters.into_iter().enumerate() {
        trial = match add_parameter::run(
            trial,
            add_parameter::Command {
                step_id: step_id.clone(),
                content,
            },
        ) {
            Ok(trial) => trial,
            Err(source) => {
                let _ = uow.rollback().await;
                return Err(Error::AddParameterDomain {
                    parameter_index,
                    source,
                });
            }
        };
    }

    // 3. パラメーターの削除
    for parameter_id in input.remove_parameter_ids {
        trial = match remove_parameter::run(
            trial,
            remove_parameter::Command {
                step_id: step_id.clone(),
                parameter_id: ParameterId(parameter_id),
            },
        ) {
            Ok(trial) => trial,
            Err(e) => {
                let _ = uow.rollback().await;
                return Err(Error::RemoveParameterDomain(e));
            }
        };
    }

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

    fn base_input(trial_id: Uuid, step_id: Uuid) -> Input {
        Input {
            trial_id,
            step_id,
            name: None,
            started_at: None,
            add_parameters: Vec::new(),
            remove_parameter_ids: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_update_step_name_success() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, step_id.0)
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
    async fn test_update_step_adds_and_removes_parameters() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            add_parameters: vec![crate::domain::models::parameter::ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            }],
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
        let parameter_id = step.parameters()[0].id().clone();

        let input = Input {
            remove_parameter_ids: vec![parameter_id.0],
            ..base_input(updated.id().0, step_id.0)
        };
        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.parameters().is_empty());
    }

    #[tokio::test]
    async fn test_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();

        let input = base_input(Uuid::new_v4(), Uuid::new_v4());

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
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, Uuid::new_v4())
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
        trial.complete(None);
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            name: Some("新名称".to_string()),
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(update_step::Error::TrialAlreadyCompleted))
        );
    }

    #[tokio::test]
    async fn test_propagates_add_parameter_domain_error_with_index() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            add_parameters: vec![
                crate::domain::models::parameter::ParameterContent::Text {
                    value: "OK".to_string(),
                },
                crate::domain::models::parameter::ParameterContent::KeyValue {
                    key: "強力粉".to_string(),
                    value: crate::domain::models::parameter::ParameterValue::Quantity {
                        amount: 300.0,
                        unit: "".to_string(),
                    },
                },
            ],
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::AddParameterDomain {
                parameter_index: 1,
                source: add_parameter::Error::InvalidParameter(
                    add_parameter::ParameterValidationError::EmptyQuantityUnit
                ),
            })
        );
    }

    #[tokio::test]
    async fn test_propagates_remove_parameter_domain_error() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            remove_parameter_ids: vec![Uuid::new_v4()],
            ..base_input(trial_id.0, step_id.0)
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::RemoveParameterDomain(
                remove_parameter::Error::ParameterNotFound
            ))
        );
    }
}
