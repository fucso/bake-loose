//! `updateTrial` mutation tests
//!
//! update_completed_trial_returns_error テストは complete_trial.rs に配置。

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_update_trial(pool: PgPool) {
    let query = r#"
        mutation {
            updateTrial(input: {
                id: "aaaa0000-0000-0000-0000-000000000001"
                name: "更新後のトライアル名"
                memo: "更新後のメモ"
            }) {
                id
                name
                memo
                status
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["updateTrial"];

    assert_eq!(trial["id"], "aaaa0000-0000-0000-0000-000000000001");
    assert_eq!(trial["name"], "更新後のトライアル名");
    assert_eq!(trial["memo"], "更新後のメモ");
    assert_eq!(trial["status"], "IN_PROGRESS");
}
