//! add_step ユースケース
//!
//! trial_id で Trial を取得し、add_step ドメインアクションを適用・保存する。

use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::add_step;
use crate::domain::models::trial::{Trial, TrialId};
use crate::domain::timezone::JstDateTime;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

/// ユースケースの入力
pub struct Input {
    pub trial_id: Uuid,
    pub name: String,
    pub started_at: Option<DateTime<FixedOffset>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(add_step::Error),
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
    let command = add_step::Command {
        name: input.name,
        started_at: input.started_at.map(JstDateTime::from_fixed_offset),
    };
    let trial = add_step::run(trial, command).map_err(Error::Domain)?;

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
    use crate::domain::actions::trial::add_step;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::Trial;
    use crate::use_case::test::MockUnitOfWork;

    fn input(trial_id: Uuid, name: &str) -> Input {
        Input {
            trial_id,
            name: name.to_string(),
            started_at: None,
        }
    }

    #[tokio::test]
    async fn test_execute_adds_step_to_existing_trial() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let result = execute(&mut uow, input(trial_id.0, "こね")).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.steps().len(), 1);
        assert_eq!(updated.steps()[0].name(), "こね");

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.steps().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let non_existing_id = Uuid::new_v4();

        let result = execute(&mut uow, input(non_existing_id, "こね")).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_already_completed() {
        let mut uow = MockUnitOfWork::default();
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete(None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let result = execute(&mut uow, input(trial_id.0, "こね")).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(add_step::Error::TrialAlreadyCompleted)
        );
    }

    /// 同一 Trial への並行 addStep で position が重複した場合、
    /// 内部エラーではなく競合エラーとして返す
    #[tokio::test]
    async fn test_execute_returns_conflict_when_save_violates_unique_constraint() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();
        // テストデータ投入後に永続化だけを一意制約違反で失敗させる
        uow.fail_save_with(RepositoryError::Conflict {
            entity: "step".to_string(),
            field: "trial_id_position".to_string(),
        });

        let result = execute(&mut uow, input(trial_id.0, "こね")).await;

        assert_eq!(result.unwrap_err(), Error::Conflict);
        // ロールバックが呼ばれ、コミットは呼ばれていないこと
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(uow.commit_count(), 0);
    }

    /// 一意制約違反以外の永続化エラーは従来どおり Infrastructure のまま
    #[tokio::test]
    async fn test_execute_returns_infrastructure_error_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();
        uow.fail_save();

        let result = execute(&mut uow, input(trial_id.0, "こね")).await;

        assert!(matches!(result, Err(Error::Infrastructure(_))));
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(uow.commit_count(), 0);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_invalid_step_name() {
        let mut uow = MockUnitOfWork::default();
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let result = execute(&mut uow, input(trial_id.0, "")).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(add_step::Error::InvalidStepName(
                add_step::StepNameError::EmptyName
            ))
        );
    }
}
