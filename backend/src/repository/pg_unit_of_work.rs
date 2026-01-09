//! PgUnitOfWork 実装
//!
//! UnitOfWork トレイトの PostgreSQL 実装。

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ports::error::RepositoryError;
use crate::ports::UnitOfWork;

use super::executor::PgExecutor;
use super::project_repo::PgProjectRepository;

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

    fn project_repository(&mut self) -> Self::ProjectRepo {
        PgProjectRepository::new(self.executor())
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
