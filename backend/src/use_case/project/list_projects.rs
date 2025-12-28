//! list_projects ユースケース
//!
//! すべてのプロジェクトを取得する。

use crate::domain::models::project::Project;
use crate::ports::{ProjectRepository, ProjectSort, UnitOfWork};

#[derive(Debug)]
pub enum Error {
    Infrastructure(String),
}

/// すべてのプロジェクトを取得する（name の昇順）
///
/// # Arguments
/// * `uow` - UnitOfWork トレイトを実装したインスタンス
///
/// # Returns
/// * `Ok(Vec<Project>)` - プロジェクト一覧（name 昇順）
/// * `Err(Error)` - エラーが発生した場合
pub async fn execute<U: UnitOfWork>(uow: &U) -> Result<Vec<Project>, Error> {
    uow.project_repository()
        .find_all(ProjectSort::default())
        .await
        .map_err(|e| Error::Infrastructure(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::use_case::test::MockUnitOfWork;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_projects_returns_sorted_by_name_asc() {
        // name の順序がバラバラなデータを用意
        let projects = vec![
            Project::from_raw(ProjectId(Uuid::new_v4()), "チーズケーキ".to_string()),
            Project::from_raw(ProjectId(Uuid::new_v4()), "アップルパイ".to_string()),
            Project::from_raw(ProjectId(Uuid::new_v4()), "バゲット".to_string()),
        ];

        let uow = MockUnitOfWork::with_projects(projects);

        let result = execute(&uow).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 3);

        // name 昇順で返ってくることを確認
        assert_eq!(found[0].name(), "アップルパイ");
        assert_eq!(found[1].name(), "チーズケーキ");
        assert_eq!(found[2].name(), "バゲット");
    }

    #[tokio::test]
    async fn test_list_projects_empty() {
        let uow = MockUnitOfWork::with_projects(vec![]);

        let result = execute(&uow).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
