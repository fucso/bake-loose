use sqlx::{postgres::PgPoolOptions, PgPool};

/// PostgreSQL 接続プールを作成する
///
/// # Arguments
/// * `url` - データベース接続URL（例: "postgres://user:pass@localhost/db"）
///
/// # Returns
/// * `Result<PgPool, sqlx::Error>` - 成功時は接続プール、失敗時はエラー
pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
}
