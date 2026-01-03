//! GraphQL エラー変換
//!
//! ユースケース層のエラーを GraphQL エラーに変換する。

use async_graphql::ErrorExtensions;

use crate::domain::actions::project::create_project as create_project_action;
use crate::use_case::project::{create_project, get_project};

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

impl UserFacingError for get_project::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            get_project::Error::Infrastructure(e) => {
                // インフラエラーの詳細は隠蔽
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<get_project::Error> for async_graphql::Error {
    fn from(e: get_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for create_project::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_project::Error::Domain(e) => match e {
                create_project_action::Error::EmptyName => {
                    GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR")
                }
                create_project_action::Error::NameTooLong { max, .. } => GraphQLError::new(
                    format!("{}文字以内で入力してください", max),
                    "VALIDATION_ERROR",
                ),
            },
            create_project::Error::DuplicateName => {
                GraphQLError::new("同じ名前のプロジェクトが既に存在します", "DUPLICATE_ERROR")
            }
            create_project::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<create_project::Error> for async_graphql::Error {
    fn from(e: create_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}
