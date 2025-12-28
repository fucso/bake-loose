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
///
/// # Arguments
/// * `uow` - UnitOfWork トレイトを実装したインスタンス
/// * `id` - 取得するプロジェクトのID
///
/// # Returns
/// * `Ok(Some(Project))` - プロジェクトが見つかった場合
/// * `Ok(None)` - プロジェクトが見つからなかった場合
/// * `Err(Error)` - エラーが発生した場合
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
        // 2件のプロジェクトを用意
        let target_id = ProjectId(Uuid::new_v4());
        let other_id = ProjectId(Uuid::new_v4());
        let target_project = Project::from_raw(target_id.clone(), "対象プロジェクト".to_string());
        let other_project = Project::from_raw(other_id.clone(), "別のプロジェクト".to_string());

        let uow = MockUnitOfWork::with_projects(vec![other_project, target_project]);

        // 対象IDで取得
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
        let existing_id = ProjectId(Uuid::new_v4());
        let non_existing_id = ProjectId(Uuid::new_v4());
        let project = Project::from_raw(existing_id, "既存プロジェクト".to_string());

        let uow = MockUnitOfWork::with_projects(vec![project]);

        // 存在しないIDで取得
        let result = execute(&uow, &non_existing_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
