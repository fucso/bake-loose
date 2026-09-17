//! `updateParameter` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{add_step, add_step_with_parameters, TRIAL_ID};

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_parameter_replaces_content_successfully(pool: PgPool) {
    let seeded = add_step_with_parameters(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = seeded["updateStep"]["id"].as_str().unwrap().to_string();
    let parameter_id = seeded["updateStep"]["parameters"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let query = format!(
        r#"
        mutation {{
            updateParameter(
                trialId: "{TRIAL_ID}"
                stepId: "{step_id}"
                parameterId: "{parameter_id}"
                content: {{ type: "text", value: "打ち粉を多めに" }}
            ) {{
                id
                content
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let parameter = &data["updateParameter"];
    assert_eq!(parameter["id"], parameter_id);
    assert_eq!(
        parameter["content"],
        json!({ "type": "text", "value": "打ち粉を多めに" })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_parameter_returns_not_found_for_missing_parameter(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let query = format!(
        r#"
        mutation {{
            updateParameter(
                trialId: "{TRIAL_ID}"
                stepId: "{step_id}"
                parameterId: "00000000-0000-0000-0000-000000000000"
                content: {{ type: "text", value: "更新" }}
            ) {{ id }}
        }}
        "#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "指定されたParameterが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_parameter_returns_error_for_content_type_mismatch(pool: PgPool) {
    let seeded = add_step_with_parameters(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = seeded["updateStep"]["id"].as_str().unwrap().to_string();
    let parameter_id = seeded["updateStep"]["parameters"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let query = format!(
        r#"
        mutation {{
            updateParameter(
                trialId: "{TRIAL_ID}"
                stepId: "{step_id}"
                parameterId: "{parameter_id}"
                content: {{ type: "duration", duration: {{ value: 90, unit: "minute" }}, note: "一次発酵" }}
            ) {{ id }}
        }}
        "#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "Parameterの種類は変更できません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}
