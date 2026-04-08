//! GraphQL エラー変換
//!
//! ユースケース層のエラーを GraphQL エラーに変換する。

use async_graphql::ErrorExtensions;

use crate::domain::actions::project::create_project as create_project_action;
use crate::use_case::project::{create_project, get_project, list_projects};
use crate::use_case::trial::{
    add_step, complete_step, complete_trial, create_trial, get_trial, list_trials_by_project,
    update_step, update_trial,
};

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

impl UserFacingError for list_projects::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            list_projects::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<list_projects::Error> for async_graphql::Error {
    fn from(e: list_projects::Error) -> Self {
        e.to_user_facing().extend()
    }
}

// --- Trial 関連エラー変換 ---

impl UserFacingError for get_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            get_trial::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<get_trial::Error> for async_graphql::Error {
    fn from(e: get_trial::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for list_trials_by_project::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            list_trials_by_project::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<list_trials_by_project::Error> for async_graphql::Error {
    fn from(e: list_trials_by_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_trial::Error::Domain(_) => {
                // 空の Error enum（将来の拡張用）
                GraphQLError::new("トライアルの作成に失敗しました", "VALIDATION_ERROR")
            }
            create_trial::Error::StepValidation(e) => match &e.error {
                crate::domain::actions::trial::add_step::Error::TrialAlreadyCompleted => {
                    // 新規作成した Trial は InProgress なので到達しないがパターンマッチのために定義
                    log::error!("Unexpected TrialAlreadyCompleted error during create_trial");
                    GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
                }
                crate::domain::actions::trial::add_step::Error::EmptyStepName => {
                    GraphQLError::new("ステップ名を入力してください", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::add_step::Error::InvalidParameter { .. } => {
                    GraphQLError::new("パラメーターが不正です", "VALIDATION_ERROR")
                }
            },
            create_trial::Error::ProjectNotFound => {
                GraphQLError::new("プロジェクトが見つかりません", "NOT_FOUND")
            }
            create_trial::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<create_trial::Error> for async_graphql::Error {
    fn from(e: create_trial::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for update_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            update_trial::Error::Domain(e) => match e {
                crate::domain::actions::trial::update_trial::Error::TrialAlreadyCompleted => {
                    GraphQLError::new("既に完了しています", "VALIDATION_ERROR")
                }
            },
            update_trial::Error::TrialNotFound => {
                GraphQLError::new("トライアルが見つかりません", "NOT_FOUND")
            }
            update_trial::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<update_trial::Error> for async_graphql::Error {
    fn from(e: update_trial::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for complete_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            complete_trial::Error::Domain(e) => match e {
                crate::domain::actions::trial::complete_trial::Error::TrialAlreadyCompleted => {
                    GraphQLError::new("既に完了しています", "VALIDATION_ERROR")
                }
            },
            complete_trial::Error::TrialNotFound => {
                GraphQLError::new("トライアルが見つかりません", "NOT_FOUND")
            }
            complete_trial::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<complete_trial::Error> for async_graphql::Error {
    fn from(e: complete_trial::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for add_step::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            add_step::Error::Domain(e) => match e {
                crate::domain::actions::trial::add_step::Error::TrialAlreadyCompleted => {
                    GraphQLError::new("既に完了しています", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::add_step::Error::EmptyStepName => {
                    GraphQLError::new("ステップ名を入力してください", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::add_step::Error::InvalidParameter { .. } => {
                    GraphQLError::new("パラメーターが不正です", "VALIDATION_ERROR")
                }
            },
            add_step::Error::TrialNotFound => {
                GraphQLError::new("トライアルが見つかりません", "NOT_FOUND")
            }
            add_step::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<add_step::Error> for async_graphql::Error {
    fn from(e: add_step::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for update_step::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            update_step::Error::Domain(e) => match e {
                crate::domain::actions::trial::update_step::Error::TrialAlreadyCompleted
                | crate::domain::actions::trial::update_step::Error::StepAlreadyCompleted => {
                    GraphQLError::new("既に完了しています", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::update_step::Error::StepNotFound => {
                    GraphQLError::new("ステップが見つかりません", "NOT_FOUND")
                }
                crate::domain::actions::trial::update_step::Error::EmptyStepName => {
                    GraphQLError::new("ステップ名を入力してください", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::update_step::Error::InvalidParameter { .. } => {
                    GraphQLError::new("パラメーターが不正です", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::update_step::Error::ParameterNotFound { .. } => {
                    GraphQLError::new("パラメーターが見つかりません", "NOT_FOUND")
                }
            },
            update_step::Error::TrialNotFound => {
                GraphQLError::new("トライアルが見つかりません", "NOT_FOUND")
            }
            update_step::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<update_step::Error> for async_graphql::Error {
    fn from(e: update_step::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for complete_step::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            complete_step::Error::Domain(e) => match e {
                crate::domain::actions::trial::complete_step::Error::TrialAlreadyCompleted
                | crate::domain::actions::trial::complete_step::Error::StepAlreadyCompleted => {
                    GraphQLError::new("既に完了しています", "VALIDATION_ERROR")
                }
                crate::domain::actions::trial::complete_step::Error::StepNotFound => {
                    GraphQLError::new("ステップが見つかりません", "NOT_FOUND")
                }
            },
            complete_step::Error::TrialNotFound => {
                GraphQLError::new("トライアルが見つかりません", "NOT_FOUND")
            }
            complete_step::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<complete_step::Error> for async_graphql::Error {
    fn from(e: complete_step::Error) -> Self {
        e.to_user_facing().extend()
    }
}
