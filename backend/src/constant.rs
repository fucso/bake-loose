//! 定数モジュール
//!
//! アプリケーション全体で使用する定数を管理する。

pub mod env;

pub use env::{get as env, load as load_env, Env, LoadError as EnvLoadError};
