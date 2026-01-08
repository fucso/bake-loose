//! list_projects ユースケース
//!
//! プロジェクト一覧を取得する。

use crate::domain::models::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::{ProjectSort, UnitOfWork};

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// プロジェクト一覧を取得する
///
/// 読み取り専用のためトランザクションは不要。
pub async fn execute<U: UnitOfWork>(uow: &mut U) -> Result<Vec<Project>, Error> {
    uow.project_repository()
        .find_all(ProjectSort::default())
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::{Project, ProjectId};
    use crate::use_case::test::MockUnitOfWork;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_projects_returns_sorted_by_name_asc() {
        let p1 = Project::from_raw(ProjectId(Uuid::new_v4()), "B Project".to_string());
        let p2 = Project::from_raw(ProjectId(Uuid::new_v4()), "A Project".to_string());
        let p3 = Project::from_raw(ProjectId(Uuid::new_v4()), "C Project".to_string());

        let mut uow = MockUnitOfWork::default();
        uow.project_repository().save(&p1).await.unwrap();
        uow.project_repository().save(&p2).await.unwrap();
        uow.project_repository().save(&p3).await.unwrap();

        let result = execute(&mut uow).await;

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 3);
        assert_eq!(projects[0].name(), "A Project");
        assert_eq!(projects[1].name(), "B Project");
        assert_eq!(projects[2].name(), "C Project");
    }

    #[tokio::test]
    async fn test_list_projects_empty() {
        let mut uow = MockUnitOfWork::default();
        let result = execute(&mut uow).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
