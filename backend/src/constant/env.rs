//! 環境変数で設定可能な値のグローバル定数
//!
//! システムで利用する環境変数を一元管理し、
//! 設定すべき環境変数を明確にする。
//!
//! 新しい環境変数を追加する場合:
//! 1. `Env` struct にフィールドを追加
//! 2. `load()` 関数内で環境変数を読み込む処理を追加
//!
//! ## 環境変数一覧
//!
//! | 変数名 | 必須 | デフォルト | 説明 |
//! |--------|------|------------|------|
//! | DATABASE_URL | Yes | - | PostgreSQL 接続URL |
//! | TEST_DATABASE_URL | No | - | テスト用 PostgreSQL 接続URL |
//! | SERVER_PORT | No | 8080 | サーバーのポート番号 |

use std::sync::OnceLock;

/// 環境変数から読み込む設定値
#[derive(Debug, Clone)]
pub struct Env {
    /// PostgreSQL 接続URL
    /// 環境変数: DATABASE_URL（必須）
    pub database_url: String,

    /// テスト用 PostgreSQL 接続URL
    /// 環境変数: TEST_DATABASE_URL（オプション）
    pub test_database_url: Option<String>,

    /// サーバーのポート番号
    /// 環境変数: SERVER_PORT（オプション、デフォルト: 8080）
    pub server_port: u16,
}

/// 環境変数読み込みエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// 必須の環境変数が設定されていない
    MissingEnv { name: &'static str },
    /// 環境変数の値が不正
    InvalidValue { name: &'static str },
}

static ENV: OnceLock<Env> = OnceLock::new();

/// 環境変数設定を取得する
///
/// # Panics
/// `load()` が呼び出される前に呼び出すとパニックする。
pub fn get() -> &'static Env {
    ENV.get()
        .expect("Env is not initialized. Call constant::env::load() first.")
}

/// 環境変数を読み込み、グローバル定数を初期化する
///
/// .env ファイルが存在する場合は環境変数として読み込む。
/// 既に初期化済みの場合は何もしない（冪等）。
pub fn load() -> Result<(), LoadError> {
    // 既に初期化済みなら何もしない
    if ENV.get().is_some() {
        return Ok(());
    }

    // .env ファイルを読み込む（存在しなくてもOK）
    let _ = dotenvy::dotenv();

    // DATABASE_URL（必須）
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| LoadError::MissingEnv { name: "DATABASE_URL" })?;

    // TEST_DATABASE_URL（オプション）
    let test_database_url = std::env::var("TEST_DATABASE_URL").ok();

    // SERVER_PORT（オプション、デフォルト: 8080）
    let server_port = match std::env::var("SERVER_PORT") {
        Ok(port_str) => port_str
            .parse::<u16>()
            .map_err(|_| LoadError::InvalidValue { name: "SERVER_PORT" })?,
        Err(_) => 8080,
    };

    let env = Env {
        database_url,
        test_database_url,
        server_port,
    };

    // 競合する可能性があるので、エラーは無視（別スレッドで初期化済み）
    let _ = ENV.set(env);

    Ok(())
}
