//! GraphQL スキーマ組み立て
//!
//! アプリケーション全体の GraphQL スキーマを構築する。

use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use sqlx::PgPool;

use super::query::ProjectQuery;

/// クエリルート
///
/// 各エンティティのクエリをマージする。
#[derive(MergedObject, Default)]
pub struct QueryRoot(ProjectQuery);

/// アプリケーション全体の GraphQL スキーマ
pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// スキーマを構築する
///
/// コンテキストに PgPool を設定し、リゾルバーで利用可能にする。
pub fn build_schema(pool: PgPool) -> AppSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(pool)
        .finish()
}
