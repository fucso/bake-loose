//! PgProjectRepository 実装

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::error::RepositoryError;
use crate::ports::project_repository::ProjectRepository;

use super::models::ProjectRow;

/// PostgreSQL 用の ProjectRepository 実装
pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    /// 新しい PgProjectRepository を作成する
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(Project::from))
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })
    }

    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError> {
        sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(Project::from).collect())
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// テスト用DBプールを取得する
    async fn get_test_pool() -> PgPool {
        let database_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    /// テスト用データを投入する
    async fn insert_test_project(pool: &PgPool, id: Uuid, name: &str) {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("Failed to insert test project");
    }

    /// テスト用データを削除する
    async fn delete_test_project(pool: &PgPool, id: Uuid) {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .expect("Failed to delete test project");
    }

    #[tokio::test]
    async fn test_find_by_id_returns_project_when_exists() {
        let pool = get_test_pool().await;
        let repo = PgProjectRepository::new(pool.clone());

        // テストデータ作成
        let test_id = Uuid::new_v4();
        let test_name = "テスト用ピザ生地";
        insert_test_project(&pool, test_id, test_name).await;

        // テスト実行
        let result = repo.find_by_id(&ProjectId(test_id)).await;

        // 検証
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert_eq!(project.id().0, test_id);
        assert_eq!(project.name(), test_name);

        // クリーンアップ
        delete_test_project(&pool, test_id).await;
    }

    #[tokio::test]
    async fn test_find_by_id_returns_none_when_not_exists() {
        let pool = get_test_pool().await;
        let repo = PgProjectRepository::new(pool);

        // 存在しないIDで検索
        let non_existent_id = Uuid::new_v4();
        let result = repo.find_by_id(&ProjectId(non_existent_id)).await;

        // 検証
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_all_returns_projects_ordered_by_created_at_desc() {
        let pool = get_test_pool().await;
        let repo = PgProjectRepository::new(pool.clone());

        // テストデータ作成（順序確認のため2件）
        let test_id1 = Uuid::new_v4();
        let test_id2 = Uuid::new_v4();
        insert_test_project(&pool, test_id1, "プロジェクト1").await;
        // 少し待って2件目を投入（created_atの差を作る）
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        insert_test_project(&pool, test_id2, "プロジェクト2").await;

        // テスト実行
        let result = repo.find_all().await;

        // 検証
        assert!(result.is_ok());
        let projects = result.unwrap();

        // 投入したデータが含まれていることを確認
        let test_projects: Vec<_> = projects
            .iter()
            .filter(|p| p.id().0 == test_id1 || p.id().0 == test_id2)
            .collect();
        assert_eq!(test_projects.len(), 2);

        // created_at DESC順なので、後に投入した test_id2 が先に来る
        assert_eq!(test_projects[0].id().0, test_id2);
        assert_eq!(test_projects[1].id().0, test_id1);

        // クリーンアップ
        delete_test_project(&pool, test_id1).await;
        delete_test_project(&pool, test_id2).await;
    }
}
