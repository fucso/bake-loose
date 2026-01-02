//! PgProjectRepository 実装

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::error::RepositoryError;
use crate::ports::project_repository::{ProjectRepository, ProjectSort};

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

    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
        // カラム名は enum から取得するので SQL インジェクションの心配なし
        let query = format!("SELECT * FROM projects {}", sort.to_order_by_clause());

        sqlx::query_as::<_, ProjectRow>(&query)
            .fetch_all(&self.pool)
            .await
            .map(|rows| rows.into_iter().map(Project::from).collect())
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE name = $1)")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })
    }

    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = NOW()
            "#,
        )
        .bind(project.id().0)
        .bind(project.name())
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| RepositoryError::Internal {
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::Project;
    use crate::ports::{ProjectSortColumn, SortDirection};
    use sqlx::PgPool;
    use uuid::Uuid;

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

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_returns_project_when_exists(pool: PgPool) {
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
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_id_returns_none_when_not_exists(pool: PgPool) {
        let repo = PgProjectRepository::new(pool);

        // 存在しないIDで検索
        let non_existent_id = Uuid::new_v4();
        let result = repo.find_by_id(&ProjectId(non_existent_id)).await;

        // 検証
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_all_with_name_asc(pool: PgPool) {
        let repo = PgProjectRepository::new(pool.clone());

        // テストデータ作成（名前順の確認）
        let test_id1 = Uuid::new_v4();
        let test_id2 = Uuid::new_v4();
        let test_id3 = Uuid::new_v4();
        insert_test_project(&pool, test_id1, "チーズケーキ").await;
        insert_test_project(&pool, test_id2, "アップルパイ").await;
        insert_test_project(&pool, test_id3, "バゲット").await;

        // テスト実行
        let sort = ProjectSort::new(ProjectSortColumn::Name, SortDirection::Asc);
        let result = repo.find_all(sort).await;

        // 検証
        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 3);

        // name ASC順: アップルパイ, チーズケーキ, バゲット
        assert_eq!(projects[0].name(), "アップルパイ");
        assert_eq!(projects[1].name(), "チーズケーキ");
        assert_eq!(projects[2].name(), "バゲット");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_all_with_created_at_desc(pool: PgPool) {
        let repo = PgProjectRepository::new(pool.clone());

        // テストデータ作成（順序確認のため2件）
        let test_id1 = Uuid::new_v4();
        let test_id2 = Uuid::new_v4();
        insert_test_project(&pool, test_id1, "プロジェクト1").await;
        // 少し待って2件目を投入（created_atの差を作る）
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        insert_test_project(&pool, test_id2, "プロジェクト2").await;

        // テスト実行
        let sort = ProjectSort::new(ProjectSortColumn::CreatedAt, SortDirection::Desc);
        let result = repo.find_all(sort).await;

        // 検証
        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 2);

        // created_at DESC順なので、後に投入した test_id2 が先に来る
        assert_eq!(projects[0].id().0, test_id2);
        assert_eq!(projects[1].id().0, test_id1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_inserts_new_project(pool: PgPool) {
        let repo = PgProjectRepository::new(pool.clone());

        let new_project = Project::new("新規プロジェクト".to_string());

        let result = repo.save(&new_project).await;
        assert!(result.is_ok());

        // find_by_id で検証
        let found = repo.find_by_id(new_project.id()).await.unwrap().unwrap();
        assert_eq!(found.name(), "新規プロジェクト");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_save_updates_existing_project(pool: PgPool) {
        let repo = PgProjectRepository::new(pool.clone());

        // 既存データ作成
        let existing_id = Uuid::new_v4();
        insert_test_project(&pool, existing_id, "更新前プロジェクト").await;
        let project_to_update = repo
            .find_by_id(&ProjectId(existing_id))
            .await
            .unwrap()
            .unwrap();

        // 更新
        let updated_project = Project::from_raw(
            project_to_update.id().clone(),
            "更新後プロジェクト".to_string(),
        );
        let result = repo.save(&updated_project).await;
        assert!(result.is_ok());

        // find_by_id で検証
        let found = repo
            .find_by_id(&ProjectId(existing_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.name(), "更新後プロジェクト");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_exists_by_name_returns_true_when_exists(pool: PgPool) {
        let repo = PgProjectRepository::new(pool.clone());

        // 既存データ作成
        insert_test_project(&pool, Uuid::new_v4(), "存在するプロジェクト").await;

        let result = repo.exists_by_name("存在するプロジェクト").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_exists_by_name_returns_false_when_not_exists(pool: PgPool) {
        let repo = PgProjectRepository::new(pool);
        let result = repo.exists_by_name("存在しないプロジェクト").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
