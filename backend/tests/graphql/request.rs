//! GraphQL テスト用リクエストユーティリティ
//!
//! GraphQL テストで共通して使用するリクエスト/レスポンス処理を提供する。

use std::time::Duration;

use axum::{body::Body, http::Request, Router};
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

use bake_loose::constant::{env, load_env};
use bake_loose::create_app;

/// テスト用の DB 接続プールを作成する
///
/// TEST_DATABASE_URL が設定されていればそちらを使用し、
/// 設定されていなければ DATABASE_URL を使用する。
pub async fn create_test_pool() -> PgPool {
    let _ = load_env();
    let database_url = env()
        .test_database_url
        .as_ref()
        .unwrap_or(&env().database_url);
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .expect("Failed to create database pool for test")
}

/// テスト用のアプリケーションを作成する
async fn create_test_app() -> Router {
    let pool = create_test_pool().await;
    create_app(pool)
}

/// GraphQL クエリを実行し、レスポンスの JSON を返す
///
/// テスト用アプリケーションの作成、リクエスト送信、レスポンスのパースを行う。
///
/// # Arguments
/// * `query` - GraphQL クエリ文字列（例: `"{ projects { id name } }"`）
///
/// # Returns
/// レスポンスボディをパースした JSON 値
pub async fn execute_graphql(query: &str) -> serde_json::Value {
    let app = create_test_app().await;
    let body = serde_json::json!({ "query": query }).to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/graphql")
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

/// GraphQL レスポンスにエラーがないことを確認する
pub fn assert_no_errors(json: &serde_json::Value) {
    assert!(
        json.get("errors").is_none(),
        "GraphQL errors: {:?}",
        json.get("errors")
    );
}

/// GraphQL レスポンスの data フィールドを取得する
pub fn get_data(json: &serde_json::Value) -> &serde_json::Value {
    json.get("data")
        .unwrap_or_else(|| panic!("Response has no data field: {:?}", json))
}
