//! `updateStep` mutation のテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};
use crate::graphql::trials::helpers::{add_step, TRIAL_ID};

/// steps.sql の未完了 Step（Parameter を2件持つ）
const IN_PROGRESS_STEP_ID: &str = "88888888-8888-8888-8888-888888888888";

/// mutation の戻り値でも、選択された Parameter が DB の実データで返ることを検証する
///
/// 書き込みに必要な範囲（`WithSteps`）だけで find すると、DB に Parameter があっても
/// `parameters: []` が返る退行が起きるため、その回帰テストとして固定する。
#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_update_step_returns_parameters_when_selected(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateStep(trialId: "{TRIAL_ID}", stepId: "{IN_PROGRESS_STEP_ID}", input: {{
                name: "新名称"
            }}) {{
                id
                name
                parameters {{ id }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(
        data,
        json!({
            "updateStep": {
                "id": IN_PROGRESS_STEP_ID,
                "name": "新名称",
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
