//! `trialsByProject` クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_empty_list_when_no_trials(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") { id } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trialsByProject": [] }));
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_only_trials_for_specified_project(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "22222222-2222-2222-2222-222222222222") { id name } }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "55555555-5555-5555-5555-555555555555",
                    "name": "Other Project Trial"
                }
            ]
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_all_trials_for_project_with_multiple_trials(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") { id name } }"#,
    )
    .await;

    // trial_repo.rs の find_all_by_project は created_at, id 順に決定的にソートするため、
    // フィクスチャの投入順（id昇順）と一致する完全なJSONで検証する
    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "name": "Test Trial 1"
                },
                {
                    "id": "44444444-4444-4444-4444-444444444444",
                    "name": "Test Trial 2"
                },
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "name": "Completed Trial"
                }
            ]
        })
    );
}
