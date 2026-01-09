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
    /// pool を直接使用（読み取り専用、またはトランザクション不要な場合）
    Pool(PgPool),
    /// トランザクションを使用（書き込み操作）
    Transaction(Arc<Mutex<Transaction<'static, Postgres>>>),
}

impl PgExecutor {
    /// pool から PgExecutor を作成
    pub fn from_pool(pool: PgPool) -> Self {
        Self::Pool(pool)
    }

    /// トランザクションから PgExecutor を作成
    pub fn from_transaction(tx: Arc<Mutex<Transaction<'static, Postgres>>>) -> Self {
        Self::Transaction(tx)
    }

    /// 単一行を取得する（存在しない場合は None）
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

    /// 複数行を取得する
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

    /// スカラー値を取得する
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

    /// クエリを実行する（INSERT/UPDATE/DELETE）
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
