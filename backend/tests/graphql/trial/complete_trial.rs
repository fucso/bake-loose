//! `completeTrial` mutation tests

use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_complete_trial(pool: PgPool) {
    let query = r#"
        mutation {
            completeTrial(id: "aaaa0000-0000-0000-0000-000000000001") {
                id
                status
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["completeTrial"];

    assert_eq!(trial["id"], "aaaa0000-0000-0000-0000-000000000001");
    assert_eq!(trial["status"], "COMPLETED");
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_update_completed_trial_returns_error(pool: PgPool) {
    // Trial 2 は completed 状態
    let query = r#"
        mutation {
            updateTrial(input: {
                id: "aaaa0000-0000-0000-0000-000000000002"
                name: "更新テスト"
            }) {
                id
            }
        }
    "#;

    let response = execute_graphql_with_errors(pool, query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "既に完了しています");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}
