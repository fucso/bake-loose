//! update_trial ユースケース
//!
//! trial_id で Trial を取得し update_trial ドメインアクションを適用・保存する。

use uuid::Uuid;

use crate::domain::actions::trial::update_trial;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

/// ユースケースの入力
///
/// name/memo は `None`: 変更なし / `Some(None)`: クリア / `Some(Some(v))`: 設定。
pub struct Input {
    pub trial_id: Uuid,
    pub name: Option<Option<String>>,
    pub memo: Option<Option<String>>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Domain(update_trial::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. Trial を取得
    let trial_id = TrialId(input.trial_id);
    let trial = match uow
        .trial_repository()
        .find_by_id(&trial_id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        Some(trial) => trial,
        None => return Err(Error::NotFound),
    };

    // 2. ドメインアクション実行
    let command = update_trial::Command {
        name: input.name,
        memo: input.memo,
    };
    let updated = match update_trial::run(trial, command) {
        Ok(trial) => trial,
        Err(e) => return Err(Error::Domain(e)),
    };

    // 3. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

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
}
