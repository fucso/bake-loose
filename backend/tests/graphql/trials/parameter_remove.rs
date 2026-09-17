//! `removeParameter` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{
    add_parameter, add_step, add_step_with_parameters, TRIAL_ID,
};

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_remove_parameter_removes_specified_parameter_only(pool: PgPool) {
    let seeded = add_step_with_parameters(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = seeded["updateStep"]["id"].as_str().unwrap().to_string();
    let text_parameter_id = seeded["updateStep"]["parameters"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let rename_query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{step_id}", input: {{ name: "発酵" }}) {{
                name
            }}
        }}
        "#
    );
    let renamed = execute_graphql(pool.clone(), &rename_query).await;
    assert_eq!(renamed["updateStep"]["name"], "発酵");

    add_parameter(
        pool.clone(),
        TRIAL_ID,
        &step_id,
        r#"{ type: "text", value: "追加メモ" }"#,
    )
    .await;

    let remove_query = format!(
        r#"
        mutation {{
            removeParameter(
                trialId: "{TRIAL_ID}"
                stepId: "{step_id}"
                parameterId: "{text_parameter_id}"
            ) {{
                name
                parameters {{ content }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &remove_query).await;

    let step = &data["removeParameter"];
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
async fn test_remove_parameter_returns_error_when_parameter_not_found(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let query = format!(
        r#"
        mutation {{
            removeParameter(
                trialId: "{TRIAL_ID}"
                stepId: "{step_id}"
                parameterId: "00000000-0000-0000-0000-000000000000"
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
