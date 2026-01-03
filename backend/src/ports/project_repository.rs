//! ProjectRepository トレイト

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::error::RepositoryError;
use crate::ports::sort::Sort;

/// プロジェクトのソート可能カラム
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectSortColumn {
    #[default]
    Name,
    CreatedAt,
    UpdatedAt,
}

/// プロジェクト一覧のソート条件
pub type ProjectSort = Sort<ProjectSortColumn>;

/// プロジェクトリポジトリのトレイト
#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    /// IDでプロジェクトを取得する
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;

    /// すべてのプロジェクトを取得する
    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError>;

    /// 指定した名前のプロジェクトが存在するかを確認する
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;

    /// プロジェクトを保存（新規作成または更新）する
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}
