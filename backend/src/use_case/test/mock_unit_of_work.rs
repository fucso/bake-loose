//! テスト用 MockUnitOfWork
//!
//! ユースケースのテストで使用する共通モック。

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::{
    ProjectRepository, ProjectSort, ProjectSortColumn, RepositoryError, SortDirection, UnitOfWork,
};
use tokio::sync::Mutex;

/// テスト用の MockProjectRepository
#[derive(Default)]
pub struct MockProjectRepository {
    // 内部可変性パターンを使用
    pub projects: Mutex<Vec<Project>>,
}

#[async_trait::async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let projects = self.projects.lock().await;
        Ok(projects.iter().find(|p| p.id() == id).cloned())
    }

    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
        let projects = self.projects.lock().await;
        let mut projects = projects.clone();

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

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let projects = self.projects.lock().await;
        Ok(projects.iter().any(|p| p.name() == name))
    }

    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        let mut projects = self.projects.lock().await;
        projects.retain(|p| p.id() != project.id());
        projects.push(project.clone());
        Ok(())
    }
}

/// テスト用の MockUnitOfWork
#[derive(Default)]
pub struct MockUnitOfWork {
    pub project_repo: MockProjectRepository,
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
