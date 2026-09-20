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
///
/// なお本番では `tx` が `None`（トランザクション外）のとき、リポジトリは
/// トランザクションではなく pool を保持するため、この Arc を掴まない。
/// そのためリポジトリへ渡すハンドルは `Option<TxHandle>` とし、
/// トランザクション中に取得したリポジトリだけがハンドルを保持する。
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
    /// トランザクションハンドル（トランザクション中に取得した場合のみ `Some`）
    ///
    /// 保持している間は commit/rollback を失敗させる。
    /// トランザクション外で取得した場合は本番の pool 相当となり `None`。
    _tx_handle: Option<TxHandle>,
}

impl MockProjectRepository {
    fn new(
        projects: Arc<Mutex<Vec<Project>>>,
        save_failure: SaveFailure,
        tx_handle: Option<TxHandle>,
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
    /// トランザクションハンドル（トランザクション中に取得した場合のみ `Some`）
    ///
    /// 保持している間は commit/rollback を失敗させる。
    /// トランザクション外で取得した場合は本番の pool 相当となり `None`。
    _tx_handle: Option<TxHandle>,
}

impl MockTrialRepository {
    fn new(
        trials: Arc<Mutex<Vec<Trial>>>,
        save_failure: SaveFailure,
        tx_handle: Option<TxHandle>,
    ) -> Self {
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
    /// リポジトリと共有する `save()` 失敗設定
    save_failure: SaveFailure,
    /// `commit()` が呼ばれた回数
    commit_count: usize,
    /// `rollback()` が呼ばれた回数
    rollback_count: usize,
    /// `rollback()` が成功した回数
    rollback_success_count: usize,
    /// 進行中のトランザクションを表すハンドル
    ///
    /// 本番の `PgUnitOfWork::tx` に対応し、`Some` の間だけトランザクション中とみなす。
    /// トランザクション状態を別のフラグで二重管理すると本番と挙動がずれるため、
    /// このフィールドだけで状態を表す。
    tx_handle: Option<TxHandle>,
}

impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            projects: Arc::new(Mutex::new(Vec::new())),
            trials: Arc::new(Mutex::new(Vec::new())),
            save_failure: Arc::new(StdMutex::new(None)),
            commit_count: 0,
            rollback_count: 0,
            rollback_success_count: 0,
            tx_handle: None,
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
        if self.tx_handle.is_some() {
            return Err(RepositoryError::Internal {
                message: "Transaction already started".to_string(),
            });
        }
        // 本番の `self.tx = Some(Arc::new(Mutex::new(tx)))` に対応する。
        // トランザクションごとに新しいハンドルを作ることで、
        // 直前のトランザクションで配ったリポジトリの生存に引きずられない。
        self.tx_handle = Some(Arc::new(()));
        Ok(())
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        self.commit_count += 1;
        // 本番は `self.tx.take()` を先に行うため、
        // この先どの経路で失敗してもトランザクション状態は解除される
        let handle = self
            .tx_handle
            .take()
            .ok_or_else(|| RepositoryError::Internal {
                message: "No transaction to commit".to_string(),
            })?;
        // PgUnitOfWork の Arc::try_unwrap 失敗と同じ条件を再現する
        if !transaction_is_free(&handle) {
            return Err(RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
            });
        }
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        self.rollback_count += 1;
        // commit と同様、本番は `self.tx.take()` を先に行う。
        // ROLLBACK の発行に失敗してもトランザクションは破棄済みなので、
        // その後の commit() は "No transaction to commit" になる
        let handle = self
            .tx_handle
            .take()
            .ok_or_else(|| RepositoryError::Internal {
                message: "No transaction to rollback".to_string(),
            })?;
        // PgUnitOfWork の Arc::try_unwrap 失敗と同じ条件を再現する。
        // リポジトリの一時値が生存したまま rollback() を呼ぶと、
        // 本番では明示的な ROLLBACK が発行されない。
        if !transaction_is_free(&handle) {
            return Err(RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
            });
        }
        self.rollback_success_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// トランザクション外で取得したリポジトリは commit を妨げない
    ///
    /// 本番では `tx` が `None` のときリポジトリは pool を保持するため、
    /// 読み取りのために取得したリポジトリを持ち続けていても
    /// `commit()` の `Arc::try_unwrap` は失敗しない。
    /// 以前のモックはハンドルを無条件に clone して渡していたため、
    /// この読み取り経路まで「Transaction is still in use」で弾いており、
    /// 本番より厳しい挙動になっていた。
    #[tokio::test]
    async fn test_commit_succeeds_while_repository_obtained_outside_transaction_is_alive() {
        let mut uow = MockUnitOfWork::default();
        // トランザクション開始前（読み取り目的）に取得したリポジトリを保持し続ける
        let read_repo = uow.trial_repository();

        uow.begin().await.unwrap();
        uow.commit().await.unwrap();

        // pool 相当のリポジトリなので commit 後も利用できる
        assert!(read_repo
            .find_by_id(&TrialId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none());
    }

    /// rollback が失敗してもトランザクション状態は解除される
    ///
    /// 本番は `self.tx.take()` を先に行うため、ROLLBACK の発行に失敗しても
    /// トランザクションは破棄され、後続の `commit()` は
    /// 「No transaction to commit」で失敗する。
    /// 以前のモックは失敗時にフラグを立てたままにしており、
    /// ロールバック失敗という再現対象そのものが本番より緩くなっていた。
    #[tokio::test]
    async fn test_failed_rollback_clears_transaction_like_production() {
        let mut uow = MockUnitOfWork::default();
        uow.begin().await.unwrap();
        // トランザクション中に取得したリポジトリを保持したまま rollback する
        let repo = uow.trial_repository();

        let rollback_error = uow.rollback().await.unwrap_err();
        assert!(matches!(
            rollback_error,
            RepositoryError::Internal { message } if message == "Transaction is still in use"
        ));
        assert_eq!(uow.rollback_count(), 1);
        assert_eq!(uow.rollback_success_count(), 0);

        drop(repo);

        // 本番ではこの時点でトランザクションは取り出し済みのため commit はできない
        let commit_error = uow.commit().await.unwrap_err();
        assert!(matches!(
            commit_error,
            RepositoryError::Internal { message } if message == "No transaction to commit"
        ));
    }

    /// トランザクション中に取得したリポジトリが生存していると commit は失敗する
    ///
    /// 本番の `Arc::try_unwrap` 失敗に対応する、モックの本来の役割。
    #[tokio::test]
    async fn test_commit_fails_while_repository_obtained_in_transaction_is_alive() {
        let mut uow = MockUnitOfWork::default();
        uow.begin().await.unwrap();
        let repo = uow.trial_repository();

        let error = uow.commit().await.unwrap_err();

        assert!(matches!(
            error,
            RepositoryError::Internal { message } if message == "Transaction is still in use"
        ));
        drop(repo);
    }
}
