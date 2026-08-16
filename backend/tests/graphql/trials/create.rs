//! `createTrial` mutation tests

use sqlx::PgPool;
use uuid::Uuid;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

const PROJECT_ID: &str = "11111111-1111-1111-1111-111111111111";

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_creates_trial_successfully(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            createTrial(input: {{ projectId: "{PROJECT_ID}", name: "焼成温度検証", memo: "初回" }}) {{
                id
                projectId
                name
                memo
                status
                steps {{ id }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let trial = &data["createTrial"];
    assert_eq!(trial["projectId"], PROJECT_ID);
    assert_eq!(trial["name"], "焼成温度検証");
    assert_eq!(trial["memo"], "初回");
    assert_eq!(trial["status"], "IN_PROGRESS");
    assert_eq!(trial["steps"], serde_json::json!([]));

    let id_str = trial["id"].as_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_creates_trial_without_name_and_memo(pool: PgPool) {
    let query = format!(
        r#"
        mutation {{
            createTrial(input: {{ projectId: "{PROJECT_ID}" }}) {{
                name
                memo
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    let trial = &data["createTrial"];
    assert_eq!(trial["name"], serde_json::Value::Null);
    assert_eq!(trial["memo"], serde_json::Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_not_found_error_for_non_existent_project(pool: PgPool) {
    let query = r#"
        mutation {
            createTrial(input: { projectId: "00000000-0000-0000-0000-000000000000", name: "test" }) {
                id
            }
        }
    "#;
    let response = execute_graphql_with_errors(pool, query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "指定されたProjectが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
    );
}
