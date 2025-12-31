//! テスト用 MockUnitOfWork
//!
//! ユースケースのテストで使用する共通モック。

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::{
    ProjectRepository, ProjectSort, ProjectSortColumn, RepositoryError, SortDirection, UnitOfWork,
};

/// テスト用の MockProjectRepository
pub struct MockProjectRepository {
    pub projects: Vec<Project>,
}

#[async_trait::async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        Ok(self.projects.iter().find(|p| p.id() == id).cloned())
    }

    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
        let mut projects = self.projects.clone();

        // ソート処理
        projects.sort_by(|a, b| {
            let cmp = match sort.column {
                ProjectSortColumn::Name => a.name().cmp(b.name()),
                // created_at, updated_at はドメインモデルにないのでテスト用に name で代用
                ProjectSortColumn::CreatedAt | ProjectSortColumn::UpdatedAt => {
                    a.name().cmp(b.name())
                }
            };
            match sort.direction {
                SortDirection::Asc => cmp,
                SortDirection::Desc => cmp.reverse(),
            }
        });

        Ok(projects)
    }
}

/// テスト用の MockUnitOfWork
pub struct MockUnitOfWork {
    pub project_repo: MockProjectRepository,
}

impl MockUnitOfWork {
    /// プロジェクト一覧から MockUnitOfWork を作成する
    pub fn with_projects(projects: Vec<Project>) -> Self {
        Self {
            project_repo: MockProjectRepository { projects },
        }
    }
}

#[async_trait::async_trait]
impl UnitOfWork for MockUnitOfWork {
    type ProjectRepo = MockProjectRepository;

    fn project_repository(&self) -> &Self::ProjectRepo {
        &self.project_repo
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }
}
