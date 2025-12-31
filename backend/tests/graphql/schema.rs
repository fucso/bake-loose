//! テスト用スキーマビルダー
//!
//! `sqlx::test` マクロから渡される `PgPool` を使用して
//! テスト用の GraphQL スキーマを構築し、クエリを実行する。

use bake_loose::presentation::graphql::build_schema;
use sqlx::PgPool;

/// GraphQL クエリを実行し、レスポンスの JSON を返す
///
/// # Arguments
/// * `pool` - `sqlx::test` から渡される PostgreSQL コネクションプール
/// * `query` - GraphQL クエリ文字列
///
/// # Returns
/// レスポンスの JSON 値
///
/// # Panics
/// GraphQL エラーが発生した場合にパニックする
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
