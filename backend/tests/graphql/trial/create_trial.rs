//! `createTrial` mutation tests

use sqlx::PgPool;
use uuid::Uuid;

use crate::graphql::schema::{execute_graphql, execute_graphql_with_errors};

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_create_trial(pool: PgPool) {
    let query = r#"
        mutation {
            createTrial(input: {
                projectId: "11111111-1111-1111-1111-111111111111"
                name: "新しいトライアル"
                memo: "テストメモ"
            }) {
                id
                projectId
                name
                memo
                status
                steps {
                    id
                }
                createdAt
                updatedAt
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["createTrial"];

    assert_eq!(trial["name"], "新しいトライアル");
    assert_eq!(trial["memo"], "テストメモ");
    assert_eq!(trial["status"], "IN_PROGRESS");
    assert_eq!(trial["projectId"], "11111111-1111-1111-1111-111111111111");
    assert!(trial["steps"].as_array().unwrap().is_empty());

    let id_str = trial["id"].as_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_trial_with_invalid_project(pool: PgPool) {
    let query = r#"
        mutation {
            createTrial(input: {
                projectId: "99999999-9999-9999-9999-999999999999"
            }) {
                id
            }
        }
    "#;

    let response = execute_graphql_with_errors(pool, query).await;

    assert_eq!(response.errors.len(), 1);
    let error = &response.errors[0];
    assert_eq!(error.message, "プロジェクトが見つかりません");
    assert_eq!(
        error.extensions.as_ref().unwrap().get("code"),
        Some(&async_graphql::Value::from("NOT_FOUND"))
    );
}
