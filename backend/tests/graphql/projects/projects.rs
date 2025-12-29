//! projects クエリのテスト
//!
//! プロジェクト一覧取得クエリのリクエストレベルテスト。

use serial_test::serial;

use crate::graphql::request::{assert_no_errors, execute_graphql, get_data};
use crate::graphql::test_data::project::{delete_test_project, insert_test_project};

#[tokio::test]
#[serial]
async fn test_returns_list() {
    let json = execute_graphql("{ projects { id name } }").await;

    assert_no_errors(&json);

    let data = get_data(&json);
    assert!(
        data.get("projects").is_some(),
        "Response should have projects field: {:?}",
        json
    );
}

#[tokio::test]
#[serial]
async fn test_contains_inserted_project() {
    // テストデータ投入
    let project = insert_test_project("test_projects_list").await;

    // クエリ実行
    let json = execute_graphql("{ projects { id name } }").await;

    assert_no_errors(&json);

    let data = get_data(&json);
    let projects = data["projects"].as_array().expect("projects should be array");

    // 投入したプロジェクトが含まれていることを確認
    let found = projects.iter().any(|p| p["name"] == project.name);
    assert!(
        found,
        "Inserted project should be in the list: {:?}",
        projects
    );

    // クリーンアップ
    delete_test_project(&project).await;
}
