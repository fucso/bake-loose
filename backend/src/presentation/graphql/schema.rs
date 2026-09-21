//! GraphQL スキーマ組み立て
//!
//! アプリケーション全体の GraphQL スキーマを構築する。

use async_graphql::{EmptySubscription, MergedObject, Schema};
use sqlx::PgPool;

use crate::presentation::graphql::mutation::project::ProjectMutation;
use crate::presentation::graphql::mutation::trial::TrialMutation;

use super::query::{ProjectQuery, TrialQuery};

/// 各エンティティのクエリをマージする。
#[derive(MergedObject, Default)]
pub struct QueryRoot(ProjectQuery, TrialQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(ProjectMutation, TrialMutation);

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

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
