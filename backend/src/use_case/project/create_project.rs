//! create_project ユースケース

use crate::domain::actions::project::create_project;
use crate::domain::models::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::unit_of_work::UnitOfWork;

/// ユースケースの入力
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub name: String,
}

/// ユースケースのエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Domain(create_project::Error),
    DuplicateName,
    Infrastructure(String),
}

/// ユースケースの実行
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Project, Error> {
    // 1. トランザクション開始
    uow.begin()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    // 2. 重複チェック
    if uow
        .project_repository()
        .exists_by_name(&input.name)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?
    {
        // エラー時はロールバック（トランザクションを開始したので）
        let _ = uow.rollback().await;
        return Err(Error::DuplicateName);
    }

    // 3. ドメインアクション実行
    let command = create_project::Command { name: input.name };
    let project = match create_project::run(command) {
        Ok(p) => p,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.project_repository().save(&project).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(format!("{:?}", e)));
    }

    // 5. コミット
    uow.commit()
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))?;

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::project::create_project;
    use crate::use_case::test::MockUnitOfWork;

    #[tokio::test]
    async fn test_execute_creates_project_successfully() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            name: "新規プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name(), "新規プロジェクト");

        // モックのリポジトリに保存されていることを確認
        let saved_project = uow
            .project_repository()
            .find_by_id(project.id())
            .await
            .unwrap();
        assert!(saved_project.is_some());
    }

    #[tokio::test]
    async fn test_execute_returns_duplicate_error_when_name_exists() {
        let mut uow = MockUnitOfWork::default();

        // 既存プロジェクトを作成（トランザクションなしで直接保存）
        let existing_project = Project::new("既存プロジェクト".to_string());
        uow.project_repository()
            .save(&existing_project)
            .await
            .unwrap();

        let input = Input {
            name: "既存プロジェクト".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::DuplicateName);
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_empty_name() {
        let mut uow = MockUnitOfWork::default();
        let input = Input {
            name: "".to_string(),
        };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(create_project::Error::EmptyName)
        );
    }

    #[tokio::test]
    async fn test_execute_returns_domain_error_for_too_long_name() {
        let mut uow = MockUnitOfWork::default();
        let long_name = "a".repeat(101);
        let input = Input { name: long_name };

        let result = execute(&mut uow, input).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Domain(create_project::Error::NameTooLong {
                max: 100,
                actual: 101
            })
        );
    }
}
