//! GraphQL コンテキストヘルパー
//!
//! リゾルバーから UnitOfWork を取得するためのヘルパー関数を提供する。

use async_graphql::{Context, Result};
use sqlx::PgPool;

use crate::repository::PgUnitOfWork;

/// コンテキストから UnitOfWork を構築する
///
/// リゾルバーはこの関数を呼び出すだけで UnitOfWork を取得できる。
/// PgPool や PgUnitOfWork の詳細はこのモジュールに隠蔽される。
pub fn create_unit_of_work(ctx: &Context<'_>) -> Result<PgUnitOfWork> {
    let pool = ctx.data::<PgPool>()?;
    Ok(PgUnitOfWork::new(pool.clone()))
}
