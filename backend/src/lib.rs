//! bake-loose バックエンドライブラリ
//!
//! アプリケーションの各モジュールを公開する。

pub mod constant;
pub mod domain;
pub mod infrastructure;
pub mod ports;
pub mod presentation;
pub mod repository;
pub mod use_case;

use axum::{routing::get, Json, Router};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

use crate::presentation::graphql::{build_schema, AppSchema};

/// ヘルスチェックのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub message: String,
}

/// アプリケーションの Router を構築する
///
/// GraphQL エンドポイント、ヘルスチェックエンドポイントを含む Router を返す。
pub fn create_app(pool: PgPool) -> Router {
    let schema = build_schema(pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/", get(health_check))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(axum::extract::Extension(schema))
        .layer(cors)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "bake-loose backend is running".to_string(),
    })
}

async fn graphql_handler(
    schema: axum::extract::Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}
