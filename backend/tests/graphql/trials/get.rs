//! `trial` クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_null_when_not_found(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trial(id: "00000000-0000-0000-0000-000000000000") { id name } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trial": null }));
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_trial(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                projectId
                name
                memo
                status
                steps { id }
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "projectId": "11111111-1111-1111-1111-111111111111",
                "name": "Test Trial 1",
                "memo": "Test Memo",
                "status": "IN_PROGRESS",
                "steps": []
            }
        })
    );
}
