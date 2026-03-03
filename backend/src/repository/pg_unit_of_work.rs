//! PgUnitOfWork 実装
//!
//! UnitOfWork トレイトの PostgreSQL 実装。

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;
use crate::ports::trial_repository::{TrialRepository, TrialSort};
use crate::ports::UnitOfWork;

use super::executor::PgExecutor;
use super::project_repo::PgProjectRepository;

/// TrialRepository の仮実装
///
/// task 05 で PgTrialRepository に差し替えられる。
pub struct PgTrialRepositoryStub;

#[async_trait]
impl TrialRepository for PgTrialRepositoryStub {
    async fn find_by_id(&self, _id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        todo!("PgTrialRepository is not yet implemented (task 05)")
    }

    async fn find_by_project_id(
        &self,
        _project_id: &ProjectId,
        _sort: TrialSort,
    ) -> Result<Vec<Trial>, RepositoryError> {
        todo!("PgTrialRepository is not yet implemented (task 05)")
    }

    async fn save(&self, _trial: &Trial) -> Result<(), RepositoryError> {
        todo!("PgTrialRepository is not yet implemented (task 05)")
    }

    async fn delete(&self, _id: &TrialId) -> Result<(), RepositoryError> {
        todo!("PgTrialRepository is not yet implemented (task 05)")
    }
}

/// PostgreSQL 用の UnitOfWork 実装
///
/// トランザクションの状態を管理し、リポジトリを提供する。
///
/// - `begin()` を呼ぶとトランザクションが開始され、以降の操作はトランザクション内で実行される
/// - `begin()` を呼ばない場合は pool を直接使用する（読み取り専用向け）
pub struct PgUnitOfWork {
    pool: PgPool,
    tx: Option<Arc<Mutex<Transaction<'static, Postgres>>>>,
}

impl PgUnitOfWork {
    /// 新しい PgUnitOfWork を作成する
    pub fn new(pool: PgPool) -> Self {
        Self { pool, tx: None }
    }

    /// 現在の Executor を取得する
    fn executor(&self) -> PgExecutor {
        match &self.tx {
            Some(tx) => PgExecutor::from_transaction(tx.clone()),
            None => PgExecutor::from_pool(self.pool.clone()),
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    type ProjectRepo = PgProjectRepository;
    type TrialRepo = PgTrialRepositoryStub;

    fn project_repository(&mut self) -> Self::ProjectRepo {
        PgProjectRepository::new(self.executor())
    }

    fn trial_repository(&mut self) -> Self::TrialRepo {
        PgTrialRepositoryStub
    }

    async fn begin(&mut self) -> Result<(), RepositoryError> {
        if self.tx.is_some() {
            return Err(RepositoryError::Internal {
                message: "Transaction already started".to_string(),
            });
        }

        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Internal {
                message: e.to_string(),
            })?;

        self.tx = Some(Arc::new(Mutex::new(tx)));
        Ok(())
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        let tx_arc = self.tx.take().ok_or_else(|| RepositoryError::Internal {
            message: "No transaction to commit".to_string(),
        })?;

        // Arc から Transaction を取り出す
        // この時点で他にこの Arc を参照しているリポジトリはないはず
        let tx = Arc::try_unwrap(tx_arc)
            .map_err(|_| RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
            })?
            .into_inner();

        tx.commit().await.map_err(|e| RepositoryError::Internal {
            message: e.to_string(),
        })
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        let tx_arc = self.tx.take().ok_or_else(|| RepositoryError::Internal {
            message: "No transaction to rollback".to_string(),
        })?;

        // Arc から Transaction を取り出す
        let tx = Arc::try_unwrap(tx_arc)
            .map_err(|_| RepositoryError::Internal {
                message: "Transaction is still in use".to_string(),
            })?
            .into_inner();

        tx.rollback().await.map_err(|e| RepositoryError::Internal {
            message: e.to_string(),
        })
    }
}
