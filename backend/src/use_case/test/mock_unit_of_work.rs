//! テスト用 MockUnitOfWork
//!
//! ユースケースのテストで使用する共通モック。

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::project_repository::ProjectRepository;
use crate::ports::{ProjectSort, ProjectSortColumn, RepositoryError, SortDirection, UnitOfWork};

/// テスト用の MockProjectRepository
///
/// MockUnitOfWork 内のデータを共有するため Arc<Mutex> を使用
#[derive(Clone)]
pub struct MockProjectRepository {
    projects: Arc<Mutex<Vec<Project>>>,
}

impl MockProjectRepository {
    fn new(projects: Arc<Mutex<Vec<Project>>>) -> Self {
        Self { projects }
    }
}

#[async_trait::async_trait]
impl ProjectRepository for MockProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let projects = self.projects.lock().await;
        Ok(projects.iter().find(|p| p.id() == id).cloned())
    }

    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
        let projects_guard = self.projects.lock().await;
        let mut projects = projects_guard.clone();

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
pub struct MockUnitOfWork {
    projects: Arc<Mutex<Vec<Project>>>,
    transaction_started: bool,
}

impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            projects: Arc::new(Mutex::new(Vec::new())),
            transaction_started: false,
        }
    }
}

#[async_trait::async_trait]
impl UnitOfWork for MockUnitOfWork {
    type ProjectRepo = MockProjectRepository;

    fn project_repository(&mut self) -> Self::ProjectRepo {
        MockProjectRepository::new(self.projects.clone())
    }

    async fn begin(&mut self) -> Result<(), RepositoryError> {
        if self.transaction_started {
            return Err(RepositoryError::Internal {
                message: "Transaction already started".to_string(),
            });
        }
        self.transaction_started = true;
        Ok(())
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        if !self.transaction_started {
            return Err(RepositoryError::Internal {
                message: "No transaction to commit".to_string(),
            });
        }
        self.transaction_started = false;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        if !self.transaction_started {
            return Err(RepositoryError::Internal {
                message: "No transaction to rollback".to_string(),
            });
        }
        self.transaction_started = false;
        Ok(())
    }
}
