//! `addParameter` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql_with_errors;
use crate::graphql::trials::helpers::{
    add_parameter, add_step, add_step_with_parameters, TRIAL_ID,
};

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_add_parameter_attaches_parameters_to_step(pool: PgPool) {
    let step = add_step_with_parameters(pool, TRIAL_ID, "こね").await;

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
async fn test_add_parameter_accepts_time_marker_content(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "焼成").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let data = add_parameter(
        pool,
        TRIAL_ID,
        &step_id,
        r#"{ type: "time_marker", at: { value: 30, unit: "minute" }, note: "温度を220度に下げる" }"#,
    )
    .await;

    // id は自動採番のため検証対象から外し、種別と内容を検証する
    let parameter = &data["addParameter"];
    assert_eq!(parameter["parameterType"], "TIME_MARKER");
    assert_eq!(
        parameter["content"],
        json!({
            "type": "time_marker",
            "at": { "value": 30.0, "unit": "minute" },
            "note": "温度を220度に下げる"
        })
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
