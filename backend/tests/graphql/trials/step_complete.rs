//! `completeStep` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{add_step, TRIAL_ID};

/// steps.sql の未完了 Step（Parameter を2件持つ）
const IN_PROGRESS_STEP_ID: &str = "88888888-8888-8888-8888-888888888888";

/// `updateStep` と同様、書き込み範囲（`WithSteps`）だけで find すると
/// `parameters` が空配列で返る退行の回帰テスト
#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_complete_step_returns_parameters_when_selected(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            completeStep(trialId: "{TRIAL_ID}", stepId: "{IN_PROGRESS_STEP_ID}") {{
                id
                isCompleted
                parameters {{ id }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(
        data,
        json!({
            "completeStep": {
                "id": IN_PROGRESS_STEP_ID,
                "isCompleted": true,
                "parameters": [
                    { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1" },
                    { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2" }
                ]
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_complete_step_successfully(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let query = format!(
        r#"mutation {{ completeStep(trialId: "{TRIAL_ID}", stepId: "{step_id}") {{ isCompleted completedAt }} }}"#
    );
    let data = execute_graphql(pool, &query).await;

    let step = &data["completeStep"];
    assert_eq!(step["isCompleted"], true);
    assert!(step["completedAt"].is_string());
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_complete_step_returns_error_when_already_completed(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let complete_query = format!(
        r#"mutation {{ completeStep(trialId: "{TRIAL_ID}", stepId: "{step_id}") {{ id }} }}"#
    );
    execute_graphql(pool.clone(), &complete_query).await;

    let response = execute_graphql_with_errors(pool, &complete_query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "Stepは既に完了しています");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}
