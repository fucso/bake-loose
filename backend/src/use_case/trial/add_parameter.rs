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
use crate::ports::{RepositoryError, UnitOfWork};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(add_parameter::Error),
    /// 一意制約違反など、並行操作との競合（リトライで解消し得る）
    Conflict,
    Infrastructure(String),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            // 一意制約違反は並行操作との競合であり、リトライで解消し得る
            RepositoryError::Conflict { .. } => Error::Conflict,
            // 外部キー違反は参照先が並行して削除されたことを意味する
            RepositoryError::NotFound { .. } => Error::NotFound,
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

/// ユースケースの入力
///
/// ParameterContent は元々オブジェクト形式の値であるため、無理にフラット化しない。
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub content: ParameterContent,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow.trial_repository().find_by_id(&trial_id).await {
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

    // 4. 永続化
    // NOTE: `if let Err(e) = uow.xxx_repository().save(..).await { .. }` と書いてはいけない。
    // if let のスクルーティニー式で作られたリポジトリの一時値は if let 文の終わりまで生存する。
    // リポジトリはトランザクションの Arc を clone して保持しているため、
    // 本体で rollback() を呼ぶ時点でも参照が残り Arc::try_unwrap が失敗して
    // 明示的な ROLLBACK が発行されなくなる。
    // そのため save() の結果をブロック内でローカルに束縛し、一時値を drop させてから判定する。
    let save_result = {
        let repo = uow.trial_repository();
        repo.save(&trial).await
    };
    if let Err(e) = save_result {
        let _ = uow.rollback().await;
        return Err(Error::from(e));
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
