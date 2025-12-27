//! Ports層
//!
//! リポジトリトレイト（インターフェース）を定義する。
//! ドメイン層とリポジトリ層の境界を抽象化する。

pub mod error;
pub mod project_repository;

pub use error::RepositoryError;
pub use project_repository::ProjectRepository;
