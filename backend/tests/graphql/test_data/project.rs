//! Project テストデータ操作
//!
//! Project のテストデータ投入・削除を行うヘルパー関数。

use uuid::Uuid;

use bake_loose::domain::models::project::ProjectId;

use crate::graphql::request::create_test_pool;

/// テストプロジェクトの情報
pub struct TestProject {
    pub id: ProjectId,
    pub name: String,
}

/// テスト用のプロジェクトを DB に投入する
///
/// 名前には UUID サフィックスが付与され、一意になる。
///
/// # Arguments
/// * `name_prefix` - プロジェクト名のプレフィックス
///
/// # Returns
/// 作成されたプロジェクトの情報
pub async fn insert_test_project(name_prefix: &str) -> TestProject {
    let pool = create_test_pool().await;
    let id = Uuid::new_v4();
    let name = format!("{}_{}", name_prefix, id);
    sqlx::query(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
    )
    .bind(id)
    .bind(&name)
    .execute(&pool)
    .await
    .expect("Failed to insert test project");
    TestProject {
        id: ProjectId(id),
        name,
    }
}

/// テスト用プロジェクトを削除する
///
/// # Arguments
/// * `project` - 削除するテストプロジェクト
pub async fn delete_test_project(project: &TestProject) {
    let pool = create_test_pool().await;
    let _ = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project.id.0)
        .execute(&pool)
        .await;
}
