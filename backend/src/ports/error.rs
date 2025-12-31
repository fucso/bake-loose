//! リポジトリ層のエラー型

/// リポジトリ操作で発生するエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// データが見つからない
    NotFound { entity: String, id: String },
    /// 一意性制約違反
    Conflict { entity: String, field: String },
    /// 接続エラー
    Connection,
    /// その他の内部エラー
    Internal { message: String },
}
