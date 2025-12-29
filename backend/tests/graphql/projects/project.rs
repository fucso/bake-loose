//! project クエリのテスト
//!
//! 単一プロジェクト取得クエリのリクエストレベルテスト。

use serial_test::serial;

use crate::graphql::request::{assert_no_errors, execute_graphql, get_data};
use crate::graphql::test_data::project::{delete_test_project, insert_test_project};

#[tokio::test]
#[serial]
async fn test_returns_null_when_not_found() {
    let json = execute_graphql(
        r#"{ project(id: "00000000-0000-0000-0000-000000000000") { id name } }"#,
    )
    .await;

    assert_no_errors(&json);

    let data = get_data(&json);
    assert!(
        data["project"].is_null(),
        "project should be null for non-existent ID: {:?}",
        json
    );
}

#[tokio::test]
#[serial]
async fn test_returns_project() {
    // テストデータ投入
    let project = insert_test_project("test_project").await;
    let other_project = insert_test_project("other_project").await;

    // クエリ実行
    let json = execute_graphql(&format!(
        r#"{{ project(id: "{}") {{ id name }} }}"#,
        project.id.0
    ))
    .await;

    assert_no_errors(&json);

    let data = get_data(&json);
    assert_eq!(data["project"]["id"], project.id.0.to_string());
    assert_eq!(data["project"]["name"], project.name);

    // クリーンアップ
    delete_test_project(&project).await;
    delete_test_project(&other_project).await;
}
