//! `completeStep` mutation のテスト

use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{add_step, TRIAL_ID};

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
