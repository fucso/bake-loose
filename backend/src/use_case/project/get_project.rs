//! get_project ユースケース
//!
//! IDでプロジェクトを取得する。

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::{ProjectRepository, UnitOfWork};

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// IDでプロジェクトを取得する
pub async fn execute<U: UnitOfWork>(uow: &U, id: &ProjectId) -> Result<Option<Project>, Error> {
    uow.project_repository()
        .find_by_id(id)
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_case::test::MockUnitOfWork;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_project_returns_specified_project_from_multiple() {
        let target_id = ProjectId(Uuid::new_v4());
        let other_id = ProjectId(Uuid::new_v4());
        let target_project = Project::from_raw(target_id.clone(), "対象プロジェクト".to_string());
        let other_project = Project::from_raw(other_id.clone(), "別のプロジェクト".to_string());

        let mut uow = MockUnitOfWork::default();
        uow.project_repository().save(&other_project).await.unwrap();
        uow.project_repository()
            .save(&target_project)
            .await
            .unwrap();

        let result = execute(&uow, &target_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());

        let project = found.unwrap();
        assert_eq!(project.id(), &target_id);
        assert_eq!(project.name(), "対象プロジェクト");
    }

    #[tokio::test]
    async fn test_get_project_not_found() {
        let project = Project::from_raw(ProjectId(Uuid::new_v4()), "既存プロジェクト".to_string());
        let mut uow = MockUnitOfWork::default();
        uow.project_repository().save(&project).await.unwrap();

        // 存在しないIDで取得
        let non_existing_id = ProjectId(Uuid::new_v4());
        let result = execute(&uow, &non_existing_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
