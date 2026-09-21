//! update_trial ユースケース
//!
//! trial_id で Trial を取得し update_trial ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::update_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

/// name/memo は `None`: 変更なし / `Some(None)`: クリア / `Some(Some(v))`: 設定。
pub struct Input {
    pub trial_id: Uuid,
    pub name: Option<Option<String>>,
    pub memo: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_trial::Error),
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

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial を取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow.trial_repository().find_by_id(&trial_id).await {
        Ok(Some(trial)) => trial,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    };

    // 2. ドメインアクション実行
    let command = update_trial::Command {
        name: input.name,
        memo: input.memo,
    };
    let updated = update_trial::run(trial, command).map_err(Error::Domain)?;

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 4. 永続化（失敗時のロールバックはヘルパー側で行う）
    save_trial(uow, &updated).await?;

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::use_case::test::MockUnitOfWork;

    fn in_progress_trial() -> Trial {
        Trial::new(
            ProjectId::new(),
            Some("元の名前".to_string()),
            Some("元のメモ".to_string()),
        )
    }

    #[tokio::test]
    async fn test_execute_updates_name_and_memo_successfully() {
        let mut uow = MockUnitOfWork::default();
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            name: Some(Some("新しい名前".to_string())),
            memo: Some(Some("新しいメモ".to_string())),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name(), Some("新しい名前"));
        assert_eq!(updated.memo(), Some("新しいメモ"));

        let saved = uow
            .trial_repository()
            .find_by_id(&trial_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.name(), Some("新しい名前"));
        assert_eq!(saved.memo(), Some("新しいメモ"));
    }

    #[tokio::test]
    async fn test_execute_returns_not_found_when_trial_does_not_exist() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            trial_id: Uuid::new_v4(),
            name: Some(Some("新しい名前".to_string())),
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::NotFound);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_when_trial_completed() {
        let mut uow = MockUnitOfWork::default();
        let mut trial = in_progress_trial();
        trial.complete(None);
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();

        let input = Input {
            trial_id: trial_id.0,
            name: Some(Some("新しい名前".to_string())),
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(
            result.unwrap_err(),
            Error::Domain(update_trial::Error::TrialAlreadyCompleted)
        );
    }

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        let trial = in_progress_trial();
        let trial_id = trial.id().clone();
        uow.trial_repository().save(&trial).await.unwrap();
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();

        let input = Input {
            trial_id: trial_id.0,
            name: Some(Some("新しい名前".to_string())),
            memo: None,
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
