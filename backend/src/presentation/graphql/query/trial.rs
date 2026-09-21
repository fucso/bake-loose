//! Trial クエリリゾルバー
//!
//! Trialに関するクエリを処理する。

use async_graphql::{Context, ErrorExtensions, Lookahead, Object, Result, ID};

use crate::ports::trial_repository::TrialScope;
use crate::presentation::graphql::common::parse_uuid;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::trial::Trial;
use crate::use_case::trial::{get_trial, list_trials_by_project};

/// selection set の look-ahead から TrialScope を組み立てる
///
/// `steps { parameters { ... } }` まで要求されていれば `Full`、
/// `steps { ... }` のみ（`parameters` 未選択）なら `WithSteps`、
/// `steps` 自体が選択されていなければ `TrialOnly` とする。
fn scope_from_look_ahead(look_ahead: Lookahead<'_>) -> TrialScope {
    let steps = look_ahead.field("steps");
    if !steps.exists() {
        return TrialScope::TrialOnly;
    }

    if steps.field("parameters").exists() {
        TrialScope::Full
    } else {
        TrialScope::WithSteps
    }
}

#[derive(Default)]
pub struct TrialQuery;

#[Object]
impl TrialQuery {
    /// IDでTrialを取得する
    ///
    /// 存在しない場合は null を返す。
    async fn trial(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Trial>> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_uuid(&id)?;
        let scope = scope_from_look_ahead(ctx.look_ahead());

        let result = get_trial::execute(&mut uow, trial_id, scope)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.map(Trial::from))
    }

    /// プロジェクトに紐づくTrial一覧を取得する
    async fn trials_by_project(&self, ctx: &Context<'_>, project_id: ID) -> Result<Vec<Trial>> {
        let mut uow = ctx.create_unit_of_work()?;

        let project_id = parse_uuid(&project_id)?;
        let scope = scope_from_look_ahead(ctx.look_ahead());

        let result = list_trials_by_project::execute(&mut uow, project_id, scope)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(result.into_iter().map(Trial::from).collect())
    }
}

// `scope_from_look_ahead` は `async_graphql::Context` 経由でしか `Lookahead` を
// 構築できないため、単体テストではなく backend/tests の GraphQL 統合テスト
// （trial_get.rs / trial_list.rs）で実際のクエリ実行を通して検証する。
// steps/parameters の要求有無に応じて正しいレイヤーのみが返ることを
// 完全な JSON 比較で確認している。
