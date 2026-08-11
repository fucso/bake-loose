//! GraphQL エラー変換
//!
//! ユースケース層のエラーを GraphQL エラーに変換する。

use async_graphql::ErrorExtensions;

use crate::domain::actions::project::create_project as create_project_action;
use crate::domain::actions::trial::{
    add_parameter as add_parameter_action, add_step as add_step_action,
    complete_step as complete_step_action, complete_trial as complete_trial_action,
    create_trial as create_trial_action, remove_parameter as remove_parameter_action,
    update_step as update_step_action, update_trial as update_trial_action,
};
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

impl UserFacingError for create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_trial::Error::Domain(create_trial_action::Error::InvalidTrialName(
                create_trial_action::TrialNameError::EmptyName,
            )) => GraphQLError::new("Trial名を入力してください", "VALIDATION_ERROR"),
            create_trial::Error::Domain(create_trial_action::Error::InvalidTrialName(
                create_trial_action::TrialNameError::NameTooLong { max, .. },
            )) => GraphQLError::new(
                format!("Trial名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
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
            update_trial::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            update_trial::Error::Domain(update_trial_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("完了済みのTrialは更新できません", "VALIDATION_ERROR")
            }
            update_trial::Error::Domain(update_trial_action::Error::InvalidTrialName(
                update_trial_action::TrialNameError::EmptyName,
            )) => GraphQLError::new("Trial名を入力してください", "VALIDATION_ERROR"),
            update_trial::Error::Domain(update_trial_action::Error::InvalidTrialName(
                update_trial_action::TrialNameError::NameTooLong { max, .. },
            )) => GraphQLError::new(
                format!("Trial名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
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
            complete_trial::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            complete_trial::Error::Domain(complete_trial_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("Trialは既に完了しています", "VALIDATION_ERROR")
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
            add_step::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            add_step::Error::Domain(add_step_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new(
                    "完了済みのTrialにはStepを追加できません",
                    "VALIDATION_ERROR",
                )
            }
            add_step::Error::Domain(add_step_action::Error::InvalidStepName(
                add_step_action::StepNameError::EmptyName,
            )) => GraphQLError::new("Step名を入力してください", "VALIDATION_ERROR"),
            add_step::Error::Domain(add_step_action::Error::InvalidStepName(
                add_step_action::StepNameError::NameTooLong { max, .. },
            )) => GraphQLError::new(
                format!("Step名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
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
            update_step::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            update_step::Error::Domain(update_step_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("完了済みのTrialのStepは更新できません", "VALIDATION_ERROR")
            }
            update_step::Error::Domain(update_step_action::Error::StepNotFound) => {
                GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
            }
            update_step::Error::Domain(update_step_action::Error::StepAlreadyCompleted) => {
                GraphQLError::new("完了済みのStepは更新できません", "VALIDATION_ERROR")
            }
            update_step::Error::Domain(update_step_action::Error::InvalidStepName(
                update_step_action::StepNameValidationError::EmptyName,
            )) => GraphQLError::new("Step名を入力してください", "VALIDATION_ERROR"),
            update_step::Error::Domain(update_step_action::Error::InvalidStepName(
                update_step_action::StepNameValidationError::NameTooLong { max, .. },
            )) => GraphQLError::new(
                format!("Step名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
            update_step::Error::AddParameterDomain {
                source: add_parameter_action::Error::TrialAlreadyCompleted,
                ..
            } => GraphQLError::new("完了済みのTrialのStepは更新できません", "VALIDATION_ERROR"),
            update_step::Error::AddParameterDomain {
                source: add_parameter_action::Error::StepNotFound,
                ..
            } => GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND"),
            update_step::Error::AddParameterDomain {
                source: add_parameter_action::Error::StepAlreadyCompleted,
                ..
            } => GraphQLError::new("完了済みのStepは更新できません", "VALIDATION_ERROR"),
            update_step::Error::AddParameterDomain {
                parameter_index,
                source:
                    add_parameter_action::Error::InvalidParameter(
                        add_parameter_action::ParameterValidationError::NegativeDurationValue,
                    ),
            } => GraphQLError::new(
                format!(
                    "{}番目のパラメーターの時間は0以上で入力してください",
                    parameter_index + 1
                ),
                "VALIDATION_ERROR",
            ),
            update_step::Error::AddParameterDomain {
                parameter_index,
                source:
                    add_parameter_action::Error::InvalidParameter(
                        add_parameter_action::ParameterValidationError::EmptyQuantityUnit,
                    ),
            } => GraphQLError::new(
                format!(
                    "{}番目のパラメーターの単位を入力してください",
                    parameter_index + 1
                ),
                "VALIDATION_ERROR",
            ),
            update_step::Error::RemoveParameterDomain(
                remove_parameter_action::Error::TrialAlreadyCompleted,
            ) => GraphQLError::new("完了済みのTrialのStepは更新できません", "VALIDATION_ERROR"),
            update_step::Error::RemoveParameterDomain(
                remove_parameter_action::Error::StepNotFound,
            ) => GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND"),
            update_step::Error::RemoveParameterDomain(
                remove_parameter_action::Error::StepAlreadyCompleted,
            ) => GraphQLError::new("完了済みのStepは更新できません", "VALIDATION_ERROR"),
            update_step::Error::RemoveParameterDomain(
                remove_parameter_action::Error::ParameterNotFound,
            ) => GraphQLError::new("指定されたParameterが見つかりません", "NOT_FOUND"),
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
            complete_step::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            complete_step::Error::Domain(complete_step_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("完了済みのTrialのStepは完了できません", "VALIDATION_ERROR")
            }
            complete_step::Error::Domain(complete_step_action::Error::StepNotFound) => {
                GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
            }
            complete_step::Error::Domain(complete_step_action::Error::StepAlreadyCompleted) => {
                GraphQLError::new("Stepは既に完了しています", "VALIDATION_ERROR")
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
