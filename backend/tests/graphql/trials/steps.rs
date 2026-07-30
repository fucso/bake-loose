//! `addStep` / `updateStep` / `completeStep` mutation tests

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

const TRIAL_ID: &str = "33333333-3333-3333-3333-333333333333";
const COMPLETED_TRIAL_ID: &str = "66666666-6666-6666-6666-666666666666";

async fn add_step(pool: PgPool, trial_id: &str, name: &str) -> serde_json::Value {
    let query = format!(
        r#"
        mutation {{
            addStep(trialId: "{trial_id}", input: {{
                name: "{name}",
                parameters: [
                    {{ type: "text", value: "打ち粉を追加" }},
                    {{ type: "key_value", key: "強力粉", value: {{ type: "quantity", amount: 300, unit: "g" }} }}
                ]
            }}) {{
                id
                name
                position
                isCompleted
                parameters {{ id content }}
            }}
        }}
        "#
    );
    execute_graphql(pool, &query).await
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_step_successfully(pool: PgPool) {
    let data = add_step(pool, TRIAL_ID, "こね").await;

    let step = &data["addStep"];
    assert_eq!(step["name"], "こね");
    assert_eq!(step["position"], 0);
    assert_eq!(step["isCompleted"], false);

    let contents: Vec<_> = step["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["content"].clone())
        .collect();
    assert_eq!(
        contents,
        vec![
            json!({ "type": "text", "value": "打ち粉を追加" }),
            json!({
                "type": "key_value",
                "key": "強力粉",
                "value": { "type": "quantity", "amount": 300.0, "unit": "g" }
            }),
        ]
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_step_returns_validation_error_for_empty_name(pool: PgPool) {
    let query =
        format!(r#"mutation {{ addStep(trialId: "{TRIAL_ID}", input: {{ name: "" }}) {{ id }} }}"#);
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "Step名を入力してください");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_step_returns_error_when_trial_completed(pool: PgPool) {
    let query = format!(
        r#"mutation {{ addStep(trialId: "{COMPLETED_TRIAL_ID}", input: {{ name: "こね" }}) {{ id }} }}"#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "完了済みのTrialにはStepを追加できません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_step_name_and_parameters(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();
    let text_parameter_id = added["addStep"]["parameters"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{step_id}", input: {{
                name: "発酵",
                addParameters: [{{ type: "text", value: "追加メモ" }}],
                removeParameterIds: ["{text_parameter_id}"]
            }}) {{
                name
                parameters {{ content }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let step = &data["updateStep"];
    assert_eq!(step["name"], "発酵");

    let contents: Vec<_> = step["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["content"].clone())
        .collect();
    assert_eq!(contents.len(), 2);
    assert!(contents.contains(&json!({ "type": "text", "value": "追加メモ" })));
    assert!(!contents
        .iter()
        .any(|c| c == &json!({ "type": "text", "value": "打ち粉を追加" })));
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_step_returns_not_found_for_missing_step(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateStep(
                trialId: "{TRIAL_ID}"
                stepId: "00000000-0000-0000-0000-000000000000"
                input: {{ name: "x" }}
            ) {{ id }}
        }}
        "#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "指定されたStepが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
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
