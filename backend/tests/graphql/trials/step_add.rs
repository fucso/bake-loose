//! `addStep` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql_with_errors;
use crate::graphql::trials::helpers::{add_step, COMPLETED_TRIAL_ID, TRIAL_ID};

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
    // addStep はパラメーターを受け付けない（追加は addParameter mutation で行う）
    assert_eq!(step["parameters"], json!([]));
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
