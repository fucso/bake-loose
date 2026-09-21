//! ProjectRepository トレイト

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::error::RepositoryError;
use crate::ports::sort::Sort;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectSortColumn {
    #[default]
    Name,
    CreatedAt,
    UpdatedAt,
}

pub type ProjectSort = Sort<ProjectSortColumn>;

#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;

    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError>;

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;

    /// プロジェクトを保存（新規作成または更新）する
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}
