//! テスト用 MockUnitOfWork
//!
//! ユースケースのテストで使用する共通モック。

use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

use crate::domain::models::project::{Project, ProjectId};
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::project_repository::ProjectRepository;
use crate::ports::trial_repository::TrialRepository;
use crate::ports::{ProjectSort, ProjectSortColumn, RepositoryError, SortDirection, UnitOfWork};

/// `save()` を失敗させるエラー設定（リポジトリと UnitOfWork で共有する）
///
/// `None` の場合は通常どおり保存される。
type SaveFailure = Arc<StdMutex<Option<RepositoryError>>>;

/// 設定されている `save()` 失敗エラーを取り出す
///
/// ロックを await をまたいで保持しないよう、同期関数として切り出している。
fn save_failure(failure: &SaveFailure) -> Option<RepositoryError> {
    failure.lock().expect("save failure lock poisoned").clone()
}

/// トランザクションハンドルの共有状態を模したマーカー
///
/// `PgUnitOfWork` はトランザクションを `Arc<Mutex<Transaction>>` で保持し、
/// `xxx_repository()` のたびにその Arc を clone してリポジトリへ渡す。
/// `commit()` / `rollback()` は `Arc::try_unwrap` でトランザクションを取り出すため、
/// リポジトリが 1 つでも生存していると `Transaction is still in use` で失敗する
/// （`repository/pg_unit_of_work.rs`）。
///
/// モックが無条件に成功していると「rollback が呼ばれるが失敗する」という
/// 本番固有の不具合を再現できないため、同型のハンドルで同じ制約をモデル化する。
type TxHandle = Arc<()>;

/// ハンドルを掴んでいるリポジトリが残っていないかを検査する
fn transaction_is_free(handle: &TxHandle) -> bool {
    Arc::strong_count(handle) == 1
}

/// テスト用の MockProjectRepository
///
/// MockUnitOfWork 内のデータを共有するため Arc<Mutex> を使用
#[derive(Clone)]
pub struct MockProjectRepository {
    projects: Arc<Mutex<Vec<Project>>>,
    /// 設定されている場合 `save()` が必ずそのエラーで失敗する
    save_failure: SaveFailure,
    /// トランザクションハンドル（生存している間は commit/rollback を失敗させる）
    _tx_handle: TxHandle,
}

impl MockProjectRepository {
    fn new(
        projects: Arc<Mutex<Vec<Project>>>,
        save_failure: SaveFailure,
        tx_handle: TxHandle,
    ) -> Self {
        Self {
            projects,
            save_failure,
            _tx_handle: tx_handle,
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
        if let Some(error) = save_failure(&self.save_failure) {
            return Err(error);
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
    /// 設定されている場合 `save()` が必ずそのエラーで失敗する
    save_failure: SaveFailure,
    /// トランザクションハンドル（生存している間は commit/rollback を失敗させる）
    _tx_handle: TxHandle,
}

impl MockTrialRepository {
    fn new(trials: Arc<Mutex<Vec<Trial>>>, save_failure: SaveFailure, tx_handle: TxHandle) -> Self {
        Self {
            trials,
            save_failure,
            _tx_handle: tx_handle,
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
        if let Some(error) = save_failure(&self.save_failure) {
            return Err(error);
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
    /// リポジトリと共有する `save()` 失敗設定
    save_failure: SaveFailure,
    /// `commit()` が呼ばれた回数
    commit_count: usize,
    /// `rollback()` が呼ばれた回数
    rollback_count: usize,
    /// `rollback()` が成功した回数
    rollback_success_count: usize,
    /// リポジトリへ clone して渡すトランザクションハンドル
    tx_handle: TxHandle,
}

impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            projects: Arc::new(Mutex::new(Vec::new())),
            trials: Arc::new(Mutex::new(Vec::new())),
            transaction_started: false,
            save_failure: Arc::new(StdMutex::new(None)),
            commit_count: 0,
            rollback_count: 0,
            rollback_success_count: 0,
            tx_handle: Arc::new(()),
        }
    }
}

impl MockUnitOfWork {
    /// 以降のすべての `save()` を内部エラーで失敗させる
    ///
    /// 永続化失敗時のロールバック経路を検証するテストで使用する。
    /// テストデータの投入後に呼び出すこと。
    pub fn fail_save(&mut self) {
        self.fail_save_with(RepositoryError::Internal {
            message: "save failed (mock)".to_string(),
        });
    }

    /// 以降のすべての `save()` を指定したエラーで失敗させる
    ///
    /// 一意制約違反（`RepositoryError::Conflict`）など、
    /// エラー種別ごとのユースケースの振る舞いを検証するテストで使用する。
    pub fn fail_save_with(&mut self, error: RepositoryError) {
        *self
            .save_failure
            .lock()
            .expect("save failure lock poisoned") = Some(error);
    }

    /// `commit()` が呼ばれた回数
    pub fn commit_count(&self) -> usize {
        self.commit_count
    }

    /// `rollback()` が呼ばれた回数
    pub fn rollback_count(&self) -> usize {
        self.rollback_count
    }

    /// `rollback()` が成功した回数
    ///
    /// 「呼ばれた」だけでは不十分で、リポジトリの一時値が生存したまま
    /// `rollback()` を呼ぶと本番では ROLLBACK が発行されない。
    /// ロールバック経路のテストではこちらをアサートすること。
    pub fn rollback_success_count(&self) -> usize {
        self.rollback_success_count
    }
}

#[async_trait::async_trait]
impl UnitOfWork for MockUnitOfWork {
    type ProjectRepo = MockProjectRepository;
    type TrialRepo = MockTrialRepository;

    fn project_repository(&mut self) -> Self::ProjectRepo {
        MockProjectRepository::new(
            self.projects.clone(),
            self.save_failure.clone(),
            self.tx_handle.clone(),
        )
    }

    fn trial_repository(&mut self) -> Self::TrialRepo {
        MockTrialRepository::new(
            self.trials.clone(),
            self.save_failure.clone(),
            self.tx_handle.clone(),
        )
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
        // PgUnitOfWork の Arc::try_unwrap 失敗と同じ条件を再現する
        if !transaction_is_free(&self.tx_handle) {
            return Err(RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
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
        // PgUnitOfWork の Arc::try_unwrap 失敗と同じ条件を再現する。
        // リポジトリの一時値が生存したまま rollback() を呼ぶと、
        // 本番では明示的な ROLLBACK が発行されない。
        if !transaction_is_free(&self.tx_handle) {
            return Err(RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
            });
        }
        self.transaction_started = false;
        self.rollback_success_count += 1;
        Ok(())
    }
}
