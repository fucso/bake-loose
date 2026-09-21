//! PgExecutor - トランザクションまたは Pool を抽象化する Executor
//!
//! リポジトリが pool 直接またはトランザクション内のどちらでも
//! 同じコードで動作するための抽象化。

use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

/// PostgreSQL の Executor を抽象化した型
///
/// `Pool` または `Transaction` のいずれかを保持し、
/// リポジトリがどちらの場合も同じインターフェースで操作できるようにする。
#[derive(Clone)]
pub enum PgExecutor {
    Pool(PgPool),
    Transaction(Arc<Mutex<Transaction<'static, Postgres>>>),
}

impl PgExecutor {
    pub fn from_pool(pool: PgPool) -> Self {
        Self::Pool(pool)
    }

    pub fn from_transaction(tx: Arc<Mutex<Transaction<'static, Postgres>>>) -> Self {
        Self::Transaction(tx)
    }

    pub async fn fetch_optional<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        match self {
            Self::Pool(pool) => query.fetch_optional(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.fetch_optional(&mut **guard).await
            }
        }
    }

    pub async fn fetch_all<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        match self {
            Self::Pool(pool) => query.fetch_all(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.fetch_all(&mut **guard).await
            }
        }
    }

    pub async fn fetch_one_scalar<'q, T>(
        &self,
        query: sqlx::query::QueryScalar<'q, Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<T, sqlx::Error>
    where
        T: Send + Unpin,
        (T,): for<'r> FromRow<'r, PgRow>,
    {
        match self {
            Self::Pool(pool) => query.fetch_one(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.fetch_one(&mut **guard).await
            }
        }
    }

    pub async fn execute<'q>(
        &self,
        query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        match self {
            Self::Pool(pool) => query.execute(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.execute(&mut **guard).await
            }
        }
    }
}
