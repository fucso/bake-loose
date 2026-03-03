//! trialsByProject クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_empty_list(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "00000000-0000-0000-0000-000000000000") { id } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trialsByProject": [] }));
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_query_trials_by_project(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") {
                id
                name
                status
            }
        }"#,
    )
    .await;

    let trials = data["trialsByProject"].as_array().unwrap();
    assert_eq!(trials.len(), 2);

    assert_eq!(trials[0]["id"], "aaaa0000-0000-0000-0000-000000000001");
    assert_eq!(trials[0]["name"], "Trial 1");
    assert_eq!(trials[0]["status"], "IN_PROGRESS");

    assert_eq!(trials[1]["id"], "aaaa0000-0000-0000-0000-000000000002");
    assert!(trials[1]["name"].is_null());
    assert_eq!(trials[1]["status"], "COMPLETED");
}
