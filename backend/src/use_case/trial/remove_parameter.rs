//! remove_parameter ユースケース
//!
//! trial_id で Trial を取得し、Step からパラメーターを削除する
//! remove_parameter ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::remove_parameter;
use crate::domain::models::parameter::ParameterId;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(remove_parameter::Error),
    /// 参照先の行が並行して削除された（外部キー違反）
    ///
    /// `entity` には参照先のエンティティ名（"trial" / "step" / "parameter" / "project"）が入る。
    /// どのエンティティが見つからないかでユーザー向けメッセージが変わるため、
    /// 集約ルートが見つからない `NotFound` とは区別する。
    ReferenceNotFound {
        entity: String,
    },
    /// 一意制約違反など、並行操作との競合（リトライで解消し得る）
    Conflict {
        entity: String,
        field: String,
    },
    Infrastructure(String),
}

impl From<RepositoryError> for Error {
    fn from(error: RepositoryError) -> Self {
        match error {
            // 一意制約違反は並行操作との競合であり、リトライで解消し得る
            RepositoryError::Conflict { entity, field } => Error::Conflict { entity, field },
            // 外部キー違反は参照先が並行して削除されたことを意味する。
            // 参照先が Trial とは限らない（例: parameters_step_id_fkey なら Step）ため、
            // エンティティ名を捨てずに presentation 層へ引き渡す
            RepositoryError::NotFound { entity, .. } => Error::ReferenceNotFound { entity },
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

/// ユースケースの入力
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub parameter_id: Uuid,
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
    let trial = remove_parameter::run(
        trial,
        remove_parameter::Command {
            step_id: StepId(input.step_id),
            parameter_id: ParameterId(input.parameter_id),
        },
    )
    .map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_trial(uow, &trial).await?;

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{Parameter, ParameterContent};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::use_case::test::MockUnitOfWork;

    fn trial_with_parameter() -> (Trial, StepId, ParameterId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        let parameter = Parameter::new(
            step_id.clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        trial.add_step(step);
        (trial, step_id, parameter_id)
    }

    #[tokio::test]
    async fn test_execute_removes_parameter_from_step() {
        let (trial, step_id, parameter_id) = trial_with_parameter();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            parameter_id: parameter_id.0,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.parameters().is_empty());

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        let saved_step = saved.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(saved_step.parameters().is_empty());
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();

        let input = Input {
            trial_id: Uuid::new_v4(),
            step_id: Uuid::new_v4(),
            parameter_id: Uuid::new_v4(),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::NotFound));
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_parameter_not_found() {
        let (trial, step_id, _parameter_id) = trial_with_parameter();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            parameter_id: Uuid::new_v4(),
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result,
            Err(Error::Domain(remove_parameter::Error::ParameterNotFound))
        );
    }

    /// 永続化に失敗した場合はロールバックし、コミットしない
    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let (trial, step_id, parameter_id) = trial_with_parameter();
        let trial_id = trial.id().clone();

        let mut uow = MockUnitOfWork::default();
        uow.trial_repository().save(&trial).await.unwrap();
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            trial_id: trial_id.0,
            step_id: step_id.0,
            parameter_id: parameter_id.0,
        };

        let result = execute(&mut uow, input).await;

        assert!(matches!(result, Err(Error::Infrastructure(_))));
        // ロールバックが実際に発行され、コミットは呼ばれていないこと
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }
}
