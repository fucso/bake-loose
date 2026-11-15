//! `updateTrial` mutation tests

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

const TRIAL_ID: &str = "33333333-3333-3333-3333-333333333333";
const COMPLETED_TRIAL_ID: &str = "66666666-6666-6666-6666-666666666666";

/// mutation の戻り値でも、選択された Step/Parameter が DB の実データで返ることを検証する
///
/// 書き込みに必要な範囲（`TrialOnly`）だけで find すると、DB に Step があっても
/// `steps: []` が返る退行が起きるため、その回帰テストとして固定する。
#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_returns_steps_and_parameters_when_selected(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateTrial(id: "{TRIAL_ID}", input: {{ name: "新しい名前" }}) {{
                name
                steps {{
                    id
                    parameters {{ id }}
                }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    assert_eq!(
        data,
        json!({
            "updateTrial": {
                "name": "新しい名前",
                "steps": [
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "parameters": [
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1" },
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2" }
                        ]
                    },
                    {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "parameters": [
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1" },
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2" }
                        ]
                    }
                ]
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_updates_name_and_memo_successfully(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateTrial(id: "{TRIAL_ID}", input: {{ name: "新しい名前", memo: "新しいメモ" }}) {{
                name
                memo
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let trial = &data["updateTrial"];
    assert_eq!(trial["name"], "新しい名前");
    assert_eq!(trial["memo"], "新しいメモ");
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_clears_memo_with_explicit_null(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateTrial(id: "{TRIAL_ID}", input: {{ memo: null }}) {{
                name
                memo
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let trial = &data["updateTrial"];
    assert_eq!(trial["name"], "Test Trial 1");
    assert_eq!(trial["memo"], serde_json::Value::Null);
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_validation_error_for_empty_name(pool: PgPool) {
    let query =
        format!(r#"mutation {{ updateTrial(id: "{TRIAL_ID}", input: {{ name: "" }}) {{ id }} }}"#);
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "Trial名を入力してください");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_validation_error_for_too_long_name(pool: PgPool) {
    let too_long_name = "あ".repeat(101);
    let query = format!(
        r#"mutation {{ updateTrial(id: "{TRIAL_ID}", input: {{ name: "{too_long_name}" }}) {{ id }} }}"#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_not_found_error(pool: PgPool) {
    let query = r#"
        mutation {
            updateTrial(id: "00000000-0000-0000-0000-000000000000", input: { name: "x" }) {
                id
            }
        }
    "#;
    let response = execute_graphql_with_errors(pool, query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "指定されたTrialが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_error_when_trial_already_completed(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            updateTrial(id: "{COMPLETED_TRIAL_ID}", input: {{ name: "x" }}) {{
                id
            }}
        }}
        "#
    );
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "完了済みのTrialは更新できません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}
