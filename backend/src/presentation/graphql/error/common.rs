//! GraphQL エラー変換の共通部品

use async_graphql::ErrorExtensions;

/// GraphQL エラーのラッパー
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQLError {
    message: String,
    code: String,
}

impl GraphQLError {
    pub fn new(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
        }
    }
}

impl ErrorExtensions for GraphQLError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(self.message.clone()).extend_with(|_err, e| {
            e.set("code", self.code.clone());
        })
    }
}

/// ユーザー向けエラーメッセージとエラーコードを拡張する
pub trait UserFacingError {
    fn to_user_facing(&self) -> GraphQLError;
}
