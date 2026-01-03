//! GraphQL スキーマ組み立て
//!
//! アプリケーション全体の GraphQL スキーマを構築する。

use async_graphql::{EmptySubscription, MergedObject, Schema};
use sqlx::PgPool;

use crate::presentation::graphql::mutation::project::ProjectMutation;

use super::query::ProjectQuery;

/// クエリルート
///
/// 各エンティティのクエリをマージする。
#[derive(MergedObject, Default)]
pub struct QueryRoot(ProjectQuery);

/// ミューテーションルート
#[derive(MergedObject, Default)]
pub struct MutationRoot(ProjectMutation);

/// アプリケーション全体の GraphQL スキーマ
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// スキーマを構築する
///
/// コンテキストに PgPool を設定し、リゾルバーで利用可能にする。
pub fn build_schema(pool: PgPool) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(pool)
    .finish()
}
