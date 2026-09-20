//! create_trial ユースケース

use uuid::Uuid;

use crate::domain::actions::trial::create_trial;
use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::{RepositoryError, UnitOfWork};

use super::save_trial;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub project_id: Uuid,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ProjectNotFound,
    Domain(create_trial::Error),
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
            // trials が持つ外部キーは project_id のみのため、参照先は必ず Project になる。
            // エンティティ名で振り分ける余地がないので ProjectNotFound に倒す
            RepositoryError::NotFound { .. } => Error::ProjectNotFound,
            other => Error::Infrastructure(format!("{:?}", other)),
        }
    }
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Trial, Error> {
    // 1. DB問い合わせが必要な検証（先に行う）
    let project_id = ProjectId(input.project_id);
    match uow.project_repository().find_by_id(&project_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(Error::ProjectNotFound),
        Err(e) => return Err(Error::Infrastructure(format!("{:?}", e))),
    }

    // 2. ドメインアクション実行
    let command = create_trial::Command {
        project_id,
        name: input.name,
        memo: input.memo,
    };
    let trial = create_trial::run(command).map_err(Error::Domain)?;

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
    use crate::domain::models::project::Project;
    use crate::ports::project_repository::ProjectRepository;
    use crate::ports::trial_repository::TrialRepository;
    use crate::use_case::test::MockUnitOfWork;

    async fn seed_project(uow: &mut MockUnitOfWork) -> ProjectId {
        let project = Project::new("テスト用プロジェクト".to_string());
        let project_id = project.id().clone();
        uow.project_repository().save(&project).await.unwrap();
        project_id
    }

    #[tokio::test]
    async fn test_execute_creates_trial_successfully() {
        let mut uow = MockUnitOfWork::default();
        let project_id = seed_project(&mut uow).await;
        let input = Input {
            project_id: project_id.0,
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
        let project_id = seed_project(&mut uow).await;
        let input = Input {
            project_id: project_id.0,
            name: None,
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let trial = result.unwrap();
        assert_eq!(trial.name(), None);
        assert_eq!(trial.memo(), None);
    }

    #[tokio::test]
    async fn test_execute_rolls_back_when_save_fails() {
        let mut uow = MockUnitOfWork::default();
        let project_id = seed_project(&mut uow).await;
        // テストデータ投入後に永続化だけを失敗させる
        uow.fail_save();
        let input = Input {
            project_id: project_id.0,
            name: Some("焼成温度検証".to_string()),
            memo: None,
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

    /// 保存時の外部キー違反（Project が並行して削除された）は
    /// 内部エラーではなく ProjectNotFound として返す
    #[tokio::test]
    async fn test_execute_returns_project_not_found_when_save_violates_foreign_key() {
        let mut uow = MockUnitOfWork::default();
        let project_id = seed_project(&mut uow).await;
        uow.fail_save_with(RepositoryError::NotFound {
            entity: "project".to_string(),
            id: "unknown".to_string(),
        });
        let input = Input {
            project_id: project_id.0,
            name: None,
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result.unwrap_err(), Error::ProjectNotFound);
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(
            uow.rollback_success_count(),
            1,
            "ROLLBACK が実際に発行されていない"
        );
        assert_eq!(uow.commit_count(), 0);
    }

    #[tokio::test]
    async fn test_execute_returns_project_not_found_for_non_existent_project() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            project_id: Uuid::new_v4(),
            name: Some("焼成温度検証".to_string()),
            memo: None,
        };

        let result = execute(&mut uow, input).await;

        assert_eq!(result, Err(Error::ProjectNotFound));
    }
}
