//! テスト用 MockUnitOfWork
//!
//! ユースケースのテストで使用する共通モック。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::models::project::{Project, ProjectId};
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::project_repository::ProjectRepository;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{ProjectSort, ProjectSortColumn, RepositoryError, SortDirection, UnitOfWork};

/// テスト用の MockProjectRepository
///
/// MockUnitOfWork 内のデータを共有するため Arc<Mutex> を使用
#[derive(Clone)]
pub struct MockProjectRepository {
    projects: Arc<Mutex<Vec<Project>>>,
    /// true の場合 `save()` が必ず失敗する（ロールバック経路の検証用）
    save_should_fail: Arc<AtomicBool>,
}

impl MockProjectRepository {
    fn new(projects: Arc<Mutex<Vec<Project>>>, save_should_fail: Arc<AtomicBool>) -> Self {
        Self {
            projects,
            save_should_fail,
        }
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
        if self.save_should_fail.load(Ordering::SeqCst) {
            return Err(RepositoryError::Internal {
                message: "save failed (mock)".to_string(),
            });
        }

        let mut projects = self.projects.lock().await;
        projects.retain(|p| p.id() != project.id());
        projects.push(project.clone());
        Ok(())
    }
}

/// テスト用の MockTrialRepository
///
/// MockUnitOfWork 内のデータを共有するため Arc<Mutex> を使用
#[derive(Clone)]
pub struct MockTrialRepository {
    trials: Arc<Mutex<Vec<Trial>>>,
    /// true の場合 `save()` が必ず失敗する（ロールバック経路の検証用）
    save_should_fail: Arc<AtomicBool>,
}

impl MockTrialRepository {
    fn new(trials: Arc<Mutex<Vec<Trial>>>, save_should_fail: Arc<AtomicBool>) -> Self {
        Self {
            trials,
            save_should_fail,
        }
    }
}

#[async_trait::async_trait]
impl TrialRepository for MockTrialRepository {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        let trials = self.trials.lock().await;
        Ok(trials.iter().find(|t| t.id() == id).cloned())
    }

    async fn find_all_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Trial>, RepositoryError> {
        let trials = self.trials.lock().await;
        Ok(trials
            .iter()
            .filter(|t| t.project_id() == project_id)
            .cloned()
            .collect())
    }

    async fn save(&self, trial: &Trial) -> Result<(), RepositoryError> {
        if self.save_should_fail.load(Ordering::SeqCst) {
            return Err(RepositoryError::Internal {
                message: "save failed (mock)".to_string(),
            });
        }

        let mut trials = self.trials.lock().await;
        trials.retain(|t| t.id() != trial.id());
        trials.push(trial.clone());
        Ok(())
    }
}

/// テスト用の MockUnitOfWork
pub struct MockUnitOfWork {
    projects: Arc<Mutex<Vec<Project>>>,
    trials: Arc<Mutex<Vec<Trial>>>,
    transaction_started: bool,
    /// リポジトリと共有する `save()` 失敗フラグ
    save_should_fail: Arc<AtomicBool>,
    /// `commit()` が呼ばれた回数
    commit_count: usize,
    /// `rollback()` が呼ばれた回数
    rollback_count: usize,
}

impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            projects: Arc::new(Mutex::new(Vec::new())),
            trials: Arc::new(Mutex::new(Vec::new())),
            transaction_started: false,
            save_should_fail: Arc::new(AtomicBool::new(false)),
            commit_count: 0,
            rollback_count: 0,
        }
    }
}

impl MockUnitOfWork {
    /// 以降のすべての `save()` を失敗させる
    ///
    /// 永続化失敗時のロールバック経路を検証するテストで使用する。
    /// テストデータの投入後に呼び出すこと。
    pub fn fail_save(&mut self) {
        self.save_should_fail.store(true, Ordering::SeqCst);
    }

    /// `commit()` が呼ばれた回数
    pub fn commit_count(&self) -> usize {
        self.commit_count
    }

    /// `rollback()` が呼ばれた回数
    pub fn rollback_count(&self) -> usize {
        self.rollback_count
    }
}

#[async_trait::async_trait]
impl UnitOfWork for MockUnitOfWork {
    type ProjectRepo = MockProjectRepository;
    type TrialRepo = MockTrialRepository;

    fn project_repository(&mut self) -> Self::ProjectRepo {
        MockProjectRepository::new(self.projects.clone(), self.save_should_fail.clone())
    }

    fn trial_repository(&mut self) -> Self::TrialRepo {
        MockTrialRepository::new(self.trials.clone(), self.save_should_fail.clone())
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
        self.commit_count += 1;
        if !self.transaction_started {
            return Err(RepositoryError::Internal {
                message: "No transaction to commit".to_string(),
            });
        }
        self.transaction_started = false;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        self.rollback_count += 1;
        if !self.transaction_started {
            return Err(RepositoryError::Internal {
                message: "No transaction to rollback".to_string(),
            });
        }
        self.transaction_started = false;
        Ok(())
    }
}
