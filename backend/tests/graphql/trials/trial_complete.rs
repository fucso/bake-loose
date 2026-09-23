//! `completeTrial` mutation tests

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

const TRIAL_ID: &str = "33333333-3333-3333-3333-333333333333";
const COMPLETED_TRIAL_ID: &str = "66666666-6666-6666-6666-666666666666";

/// `updateTrial` と同様、書き込み範囲（`TrialOnly`）だけで find すると
/// `steps` が空配列で返る退行の回帰テスト
#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_returns_steps_and_parameters_when_selected(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            completeTrial(id: "{TRIAL_ID}") {{
                status
                steps {{
                    id
                    parameters {{ id }}
                }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(
        data,
        json!({
            "completeTrial": {
                "status": "COMPLETED",
                "steps": [
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "parameters": [
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1" },
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2" }
                        ]
                    },
                    {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "parameters": [
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1" },
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2" }
                        ]
                    }
                ]
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_completes_trial_successfully(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            completeTrial(id: "{TRIAL_ID}") {{
                status
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(data["completeTrial"]["status"], "COMPLETED");
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_not_found_error(pool: PgPool) {
    let query = r#"
        mutation {
            completeTrial(id: "00000000-0000-0000-0000-000000000000") {
                id
            }
        }
    "#;
    let response = execute_graphql_with_errors(pool, query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "指定されたTrialが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_error_when_already_completed(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            completeTrial(id: "{COMPLETED_TRIAL_ID}") {{
                id
            }}
        }}
        "#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "Trialは既に完了しています");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}
