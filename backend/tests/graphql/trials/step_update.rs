//! `updateStep` mutation のテスト

use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{add_step, TRIAL_ID};

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
async fn test_update_step_sets_started_at(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{step_id}", input: {{
                startedAt: "2026-01-01T09:00:00+09:00"
            }}) {{
                startedAt
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(data["updateStep"]["startedAt"], "2026-01-01T09:00:00+09:00");
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_update_step_clears_started_at(pool: PgPool) {
    let added = add_step(pool.clone(), TRIAL_ID, "こね").await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    let set_query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{step_id}", input: {{
                startedAt: "2026-01-01T09:00:00+09:00"
            }}) {{ startedAt }}
        }}
        "#
    );
    execute_graphql(pool.clone(), &set_query).await;

    let clear_query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{step_id}", input: {{
                startedAt: null
            }}) {{ startedAt }}
        }}
        "#
    );
    let data = execute_graphql(pool, &clear_query).await;

    assert_eq!(data["updateStep"]["startedAt"], serde_json::Value::Null);
}
