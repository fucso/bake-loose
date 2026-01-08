//! `createProject` mutation tests

use sqlx::PgPool;
use uuid::Uuid;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

fn build_mutation(name: &str) -> String {
    format!(
        r#"
        mutation {{
            createProject(input: {{ name: "{}" }}) {{
                id
                name
            }}
        }}
    "#,
        name
    )
}

#[sqlx::test(migrations = "./migrations")]
async fn test_creates_project_successfully(pool: PgPool) {
    let query = build_mutation("新規プロジェクト");
    let data = execute_graphql(pool, &query).await;

    // レスポンス検証
    let project = &data["createProject"];
    assert_eq!(project["name"], "新規プロジェクト");

    // id が UUID 形式であることを検証
    let id_str = project["id"].as_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_error_for_empty_name(pool: PgPool) {
    let query = build_mutation("");
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "プロジェクト名を入力してください");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_error_for_too_long_name(pool: PgPool) {
    let long_name = "a".repeat(101);
    let query = build_mutation(&long_name);
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "100文字以内で入力してください");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("VALIDATION_ERROR"))
    );
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_error_for_duplicate_name(pool: PgPool) {
    let query = build_mutation("Test Project 1");
    let response = execute_graphql_with_errors(pool, &query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "同じ名前のプロジェクトが既に存在します");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("DUPLICATE_ERROR"))
    );
}
