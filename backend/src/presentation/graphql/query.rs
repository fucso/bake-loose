//! GraphQL クエリリゾルバー
//!
//! 各エンティティのクエリリゾルバーを提供する。

pub mod project;
pub mod trial;

pub use project::ProjectQuery;
pub use trial::TrialQuery;
