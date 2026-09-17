//! `addParameter` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql_with_errors;
use crate::graphql::trials::helpers::{add_step_with_parameters, TRIAL_ID};

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_parameter_attaches_parameters_to_step(pool: PgPool) {
    let data = add_step_with_parameters(pool, TRIAL_ID, "こね").await;

    let step = &data["updateStep"];
    assert_eq!(step["name"], "こね");

    // id は自動採番のため検証対象から外し、種別と内容を検証する
    let parameters: Vec<_> = step["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| json!({ "parameterType": p["parameterType"], "content": p["content"] }))
        .collect();
    assert_eq!(
        parameters,
        vec![
            json!({
                "parameterType": "TEXT",
                "content": { "type": "text", "value": "打ち粉を追加" }
            }),
            json!({
                "parameterType": "KEY_VALUE",
                "content": {
                    "type": "key_value",
                    "key": "強力粉",
                    "value": { "type": "quantity", "amount": 300.0, "unit": "g" }
                }
            }),
        ]
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_parameter_returns_error_when_step_not_found(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            addParameter(
                trialId: "{TRIAL_ID}"
                stepId: "00000000-0000-0000-0000-000000000000"
                content: {{ type: "text", value: "追加メモ" }}
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
