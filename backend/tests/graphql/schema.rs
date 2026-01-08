//! テスト用スキーマビルダー
//!
//! `sqlx::test` マクロから渡される `PgPool` を使用して
//! テスト用の GraphQL スキーマを構築し、クエリを実行する。

use bake_loose::presentation::graphql::build_schema;
use sqlx::PgPool;

/// GraphQL クエリを実行し、レスポンスの JSON を返す
pub async fn execute_graphql(pool: PgPool, query: &str) -> serde_json::Value {
    let schema = build_schema(pool);
    let response = schema.execute(query).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    response.data.into_json().unwrap()
}

/// GraphQL クエリを実行し、エラーを含むレスポンスを返す
pub async fn execute_graphql_with_errors(pool: PgPool, query: &str) -> async_graphql::Response {
    let schema = build_schema(pool);
    schema.execute(query).await
}
