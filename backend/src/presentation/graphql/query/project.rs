//! Project クエリリゾルバー
//!
//! プロジェクトに関するクエリを処理する。

use async_graphql::{Context, Object, Result, ID};
use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::use_case::project::{get_project, list_projects};

use super::super::context::create_unit_of_work;
use super::super::types::Project;

/// Project クエリリゾルバー
#[derive(Default)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    /// IDでプロジェクトを取得する
    ///
    /// 存在しない場合は null を返す。
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let uow = create_unit_of_work(ctx)?;

        // ID のパース
        let uuid = Uuid::parse_str(&id.0)
            .map_err(|_| async_graphql::Error::new("Invalid project ID format"))?;
        let project_id = ProjectId(uuid);

        // ユースケース実行
        let result = get_project::execute(&uow, &project_id)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{:?}", e)))?;

        Ok(result.map(Project::from))
    }

    /// すべてのプロジェクトを取得する
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let uow = create_unit_of_work(ctx)?;

        // ユースケース実行
        let result = list_projects::execute(&uow)
            .await
            .map_err(|e| async_graphql::Error::new(format!("{:?}", e)))?;

        Ok(result.into_iter().map(Project::from).collect())
    }
}
