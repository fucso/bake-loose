//! GraphQL Context 拡張
//!
//! Context にヘルパー関数を追加する。

use async_graphql::{Context, Result};
use sqlx::PgPool;

use crate::repository::PgUnitOfWork;

/// Context に `PgUnitOfWork` を作成するヘルパーを追加
pub trait ContextExt {
    fn create_unit_of_work(&self) -> Result<PgUnitOfWork>;
}

impl ContextExt for Context<'_> {
    fn create_unit_of_work(&self) -> Result<PgUnitOfWork> {
        let pool = self.data::<PgPool>()?;
        Ok(PgUnitOfWork::new(pool.clone()))
    }
}
