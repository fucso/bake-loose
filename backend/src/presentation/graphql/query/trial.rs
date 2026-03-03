//! Trial クエリリゾルバー
//!
//! トライアルに関するクエリを処理する。

use async_graphql::{Context, ErrorExtensions, Object, Result, ID};
use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::TrialId;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::trial::Trial;
use crate::use_case::trial::{get_trial, list_trials_by_project};

/// Trial クエリリゾルバー
#[derive(Default)]
pub struct TrialQuery;

#[Object]
impl TrialQuery {
    /// IDでトライアルを取得する
    ///
    /// 存在しない場合は null を返す。
    async fn trial(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Trial>> {
        let mut uow = ctx.create_unit_of_work()?;

        let uuid = Uuid::parse_str(&id.0)
            .map_err(|_| async_graphql::Error::new("Invalid trial ID format"))?;
        let trial_id = TrialId(uuid);

        let result = get_trial::execute(&mut uow, &trial_id)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.map(Trial::from))
    }

    /// プロジェクトに紐づくトライアル一覧を取得する
    async fn trials_by_project(&self, ctx: &Context<'_>, project_id: ID) -> Result<Vec<Trial>> {
        let mut uow = ctx.create_unit_of_work()?;

        let uuid = Uuid::parse_str(&project_id.0)
            .map_err(|_| async_graphql::Error::new("Invalid project ID format"))?;
        let project_id = ProjectId(uuid);

        let result = list_trials_by_project::execute(&mut uow, &project_id)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.into_iter().map(Trial::from).collect())
    }
}
