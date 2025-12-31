//! project クエリのテスト
//!
//! 単一プロジェクト取得クエリのリクエストレベルテスト。

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_null_when_not_found(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ project(id: "00000000-0000-0000-0000-000000000000") { id name } }"#,
    )
    .await;

    assert_eq!(data, json!({ "project": null }));
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_project(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ project(id: "11111111-1111-1111-1111-111111111111") { id name } }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "project": {
                "id": "11111111-1111-1111-1111-111111111111",
                "name": "Test Project 1"
            }
        })
    );
}
