//! projects クエリのテスト
//!
//! プロジェクト一覧取得クエリのリクエストレベルテスト。

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_empty_list(pool: PgPool) {
    let data = execute_graphql(pool, "{ projects { id name } }", json!({})).await;

    assert_eq!(data, json!({ "projects": [] }));
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_projects_from_fixture(pool: PgPool) {
    let data = execute_graphql(pool, "{ projects { id name } }", json!({})).await;

    assert_eq!(
        data,
        json!({
            "projects": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "name": "Test Project 1"
                },
                {
                    "id": "22222222-2222-2222-2222-222222222222",
                    "name": "Test Project 2"
                }
            ]
        })
    );
}
