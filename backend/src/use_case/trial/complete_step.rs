//! complete_step ユースケース

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::complete_step;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub trial_id: Uuid,
    pub step_id: Uuid,
    pub completed_at: Option<DateTime<FixedOffset>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(complete_step::Error),
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

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow.trial_repository().find_by_id(&trial_id).await {
        Ok(Some(trial)) => trial,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    };

    // 2. ドメインアクション実行
    let command = complete_step::Command {
        step_id: StepId(input.step_id),
        completed_at: input.completed_at.map(JstDateTime::from_fixed_offset),
    };
    let trial = complete_step::run(trial, command).map_err(Error::Domain)?;

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
            trial_id: trial.id().0,
            step_id: step_id.0,
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
            trial_id: Uuid::new_v4(),
            step_id: Uuid::new_v4(),
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
            trial_id: trial.id().0,
            step_id: Uuid::new_v4(),
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
            trial_id: trial.id().0,
            step_id: step_id.0,
            completed_at: None,
        };
        execute(&mut uow, input).await.unwrap();

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
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
        trial.complete(None);
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(complete_step::Error::TrialAlreadyCompleted)
        );
    }

    /// 永続化に失敗した場合はロールバックし、コミットしない
    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        let (trial, step_id) = seed_trial_with_step(&mut uow).await;
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            trial_id: trial.id().0,
            step_id: step_id.0,
            completed_at: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
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
