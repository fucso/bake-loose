//! Presentation層
//!
//! GraphQLリゾルバー・スキーマを担当する。

pub mod graphql;

pub use graphql::{build_schema, AppSchema};
