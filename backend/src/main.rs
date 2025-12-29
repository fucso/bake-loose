//! bake-loose バックエンド エントリーポイント
//!
//! サーバーの起動処理を行う。

use std::net::SocketAddr;

use bake_loose::constant::{env, load_env, EnvLoadError};
use bake_loose::infrastructure::database;
use bake_loose::create_app;

fn env_load_error_message(e: &EnvLoadError) -> String {
    match e {
        EnvLoadError::MissingEnv { name } => {
            format!("Required environment variable '{}' is not set", name)
        }
        EnvLoadError::InvalidValue { name } => {
            format!("Invalid value for environment variable '{}'", name)
        }
    }
}

#[tokio::main]
async fn main() {
    // ロギングの初期化
    tracing_subscriber::fmt::init();

    // 環境変数の読み込み
    if let Err(e) = load_env() {
        tracing::error!("Failed to load environment: {}", env_load_error_message(&e));
        std::process::exit(1);
    }

    // DB接続プールの作成
    let pool = match database::create_pool(&env().database_url).await {
        Ok(pool) => {
            tracing::info!("Database connection pool created");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to create database pool: {}", e);
            std::process::exit(1);
        }
    };

    // アプリケーションの構築
    let app = create_app(pool);

    // サーバー起動
    let addr = SocketAddr::from(([0, 0, 0, 0], env().server_port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
