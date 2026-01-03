//! ProjectMutation リゾルバー

use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::{project::CreateProjectInput, project::Project};
use crate::use_case::project::create_project;

/// プロジェクト関連のミューテーション
#[derive(Default)]
pub struct ProjectMutation;

#[Object]
impl ProjectMutation {
    /// プロジェクトを作成する
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let mut uow = ctx.create_unit_of_work()?;
        let input = create_project::Input { name: input.name };

        let project = create_project::execute(&mut uow, input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(project.into())
    }
}
