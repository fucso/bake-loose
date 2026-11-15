//! add_parameter ユースケース
//!
//! trial_id で Trial を取得し、Step にパラメーターを追加する
//! add_parameter ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::add_parameter;
use crate::domain::models::parameter::ParameterContent;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::{TrialRepository, TrialScope};
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(add_parameter::Error),
    ReferenceNotFound { entity: String },
    Conflict { entity: String, field: String },
    Infrastructure(String),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Conflict { entity, field } => Error::Conflict { entity, field },
            // 参照先が Trial とは限らない（例: parameters_step_id_fkey なら Step）ため、
            // エンティティ名を捨てずに presentation 層へ引き渡す
            RepositoryError::NotFound { entity, .. } => Error::ReferenceNotFound { entity },
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

/// ParameterContent は元々オブジェクト形式の値であるため、無理にフラット化しない。
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub content: ParameterContent,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial取得
    // Parameter を操作するため Full が必要
    // （他 Step の Parameter が未取得だと save の差分削除で消失する）
    let trial_id = TrialId(input.trial_id);
    let trial = match uow
        .trial_repository()
        .find_by_id(&trial_id, TrialScope::Full)
        .await
    {
        Ok(Some(trial)) => trial,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    };

    // 2. ドメインアクション実行
    let trial = add_parameter::run(
        trial,
        add_parameter::Command {
            step_id: StepId(input.step_id),
            content: input.content,
        },
    )
    .map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_trial(uow, &trial, TrialScope::Full).await?;

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
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

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
            .find_by_id(&trial_id, TrialScope::Full)
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
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

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
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();

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

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let (trial, step_id) = trial_with_step();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository()
            .save(&trial, TrialScope::Full)
            .await
            .unwrap();
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = execute(&mut uow, input).await;

        assert!(matches!(result, Err(Error::Infrastructure(_))));
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }
}
