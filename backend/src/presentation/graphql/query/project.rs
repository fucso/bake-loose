//! Project クエリリゾルバー
//!
//! プロジェクトに関するクエリを処理する。

use async_graphql::{Context, ErrorExtensions, Object, Result, ID};

use crate::domain::models::project::ProjectId;
use crate::presentation::graphql::common::parse_uuid;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::project::Project;
use crate::use_case::project::{get_project, list_projects};

/// Project クエリリゾルバー
#[derive(Default)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    /// IDでプロジェクトを取得する
    ///
    /// 存在しない場合は null を返す。
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let mut uow = ctx.create_unit_of_work()?;

        // ID のパース
        let project_id = ProjectId(parse_uuid(&id)?);

        // ユースケース実行
        let result = get_project::execute(&mut uow, &project_id)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.map(Project::from))
    }

    /// すべてのプロジェクトを取得する
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let mut uow = ctx.create_unit_of_work()?;

        // ユースケース実行
        let result = list_projects::execute(&mut uow)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.into_iter().map(Project::from).collect())
    }
}
