//! ProjectRepository トレイト

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::error::RepositoryError;

/// プロジェクトリポジトリのトレイト
#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    /// IDでプロジェクトを取得する
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;

    /// すべてのプロジェクトを取得する
    async fn find_all(&self) -> Result<Vec<Project>, RepositoryError>;
}
