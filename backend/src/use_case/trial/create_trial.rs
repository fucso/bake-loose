//! create_trial ユースケース

use crate::domain::actions::trial::create_trial;
use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::UnitOfWork;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(create_trial::Error),
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. ドメインアクション実行
    let command = create_trial::Command {
        project_id: input.project_id,
        name: input.name,
        memo: input.memo,
    };
    let trial = match create_trial::run(command) {
        Ok(t) => t,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 3. 永続化
    if let Err(e) = uow.trial_repository().save(&trial).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 4. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(trial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_execute_creates_trial_successfully() {
        let mut uow = MockUnitOfWork::default();
        let project_id = ProjectId::new();
        let input = Input {
            project_id: project_id.clone(),
            name: Some("焼成温度検証".to_string()),
            memo: Some("初回".to_string()),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        assert_eq!(trial.project_id(), &project_id);
        assert_eq!(trial.name(), Some("焼成温度検証"));
        assert_eq!(trial.memo(), Some("初回"));

        // モックのリポジトリに保存されていることを確認
        let saved_trial = uow.trial_repository().find_by_id(trial.id()).await.unwrap();
        assert!(saved_trial.is_some());
    }

    #[tokio::test]
    async fn test_execute_allows_no_name_and_memo() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            project_id: ProjectId::new(),
            name: None,
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        assert_eq!(trial.name(), None);
        assert_eq!(trial.memo(), None);
    }
}
