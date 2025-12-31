//! テスト用スキーマビルダー
//!
//! `sqlx::test` マクロから渡される `PgPool` を使用して
//! テスト用の GraphQL スキーマを構築する。

use bake_loose::presentation::graphql::{build_schema, AppSchema};
use sqlx::PgPool;

/// テスト用スキーマを構築する
///
/// # Arguments
/// * `pool` - `sqlx::test` から渡される PostgreSQL コネクションプール
///
/// # Returns
/// テスト用の `AppSchema`
pub fn create_test_schema(pool: PgPool) -> AppSchema {
    build_schema(pool)
}
