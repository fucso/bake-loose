//! Trial 関連のユースケースエラー -> GraphQL エラー変換

use async_graphql::ErrorExtensions;

use crate::domain::actions::trial::{
    add_parameter as add_parameter_action, add_step as add_step_action,
    complete_step as complete_step_action, complete_trial as complete_trial_action,
    create_trial as create_trial_action, remove_parameter as remove_parameter_action,
    update_parameter as update_parameter_action, update_step as update_step_action,
    update_trial as update_trial_action,
};
use crate::presentation::graphql::error::common::{GraphQLError, UserFacingError};
use crate::use_case::trial::{
    add_parameter, add_step, complete_step, complete_trial, create_trial, get_trial,
    list_trials_by_project, remove_parameter, update_parameter, update_step, update_trial,
};

/// Infrastructure エラーをログに残しつつ共通の内部エラーへ変換する
///
/// 複数のユースケースエラーで同一の変換ロジックを繰り返さないための共通部品。
fn internal_error(e: &str) -> GraphQLError {
    log::error!("Infrastructure error: {}", e);
    GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
}

/// 「指定されたTrialが見つかりません」エラー
///
/// 複数のユースケースエラーで同一のメッセージ・コードを繰り返さないための共通部品。
fn trial_not_found() -> GraphQLError {
    GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
}

/// 「指定されたStepが見つかりません」エラー
fn step_not_found() -> GraphQLError {
    GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
}

/// 「指定されたParameterが見つかりません」エラー
fn parameter_not_found() -> GraphQLError {
    GraphQLError::new("指定されたParameterが見つかりません", "NOT_FOUND")
}

impl UserFacingError for create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_trial::Error::ProjectNotFound => {
                GraphQLError::new("指定されたProjectが見つかりません", "NOT_FOUND")
            }
            create_trial::Error::Domain(create_trial_action::Error::InvalidTrialName(
                create_trial_action::TrialNameError::EmptyName,
            )) => GraphQLError::new("Trial名を入力してください", "VALIDATION_ERROR"),
            create_trial::Error::Domain(create_trial_action::Error::InvalidTrialName(
                create_trial_action::TrialNameError::NameTooLong { max, .. },
            )) => GraphQLError::new(
                format!("Trial名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
            create_trial::Error::Infrastructure(e) => internal_error(e),
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
            update_trial::Error::NotFound => trial_not_found(),
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
            update_trial::Error::Infrastructure(e) => internal_error(e),
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
            complete_trial::Error::NotFound => trial_not_found(),
            complete_trial::Error::Domain(complete_trial_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("Trialは既に完了しています", "VALIDATION_ERROR")
            }
            complete_trial::Error::Infrastructure(e) => internal_error(e),
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
            add_step::Error::NotFound => trial_not_found(),
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
            add_step::Error::Infrastructure(e) => internal_error(e),
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
            update_step::Error::NotFound => trial_not_found(),
            update_step::Error::Domain(update_step_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("完了済みのTrialのStepは更新できません", "VALIDATION_ERROR")
            }
            update_step::Error::Domain(update_step_action::Error::StepNotFound) => step_not_found(),
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
            update_step::Error::Infrastructure(e) => internal_error(e),
        }
    }
}

impl From<update_step::Error> for async_graphql::Error {
    fn from(e: update_step::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for add_parameter::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            add_parameter::Error::NotFound => trial_not_found(),
            add_parameter::Error::Domain(add_parameter_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new(
                    "完了済みのTrialのStepにはParameterを追加できません",
                    "VALIDATION_ERROR",
                )
            }
            add_parameter::Error::Domain(add_parameter_action::Error::StepNotFound) => {
                step_not_found()
            }
            add_parameter::Error::Domain(add_parameter_action::Error::StepAlreadyCompleted) => {
                GraphQLError::new(
                    "完了済みのStepにはParameterを追加できません",
                    "VALIDATION_ERROR",
                )
            }
            add_parameter::Error::Domain(add_parameter_action::Error::InvalidParameter(
                add_parameter_action::ParameterValidationError::NegativeDurationValue,
            )) => GraphQLError::new("時間は0以上で入力してください", "VALIDATION_ERROR"),
            add_parameter::Error::Domain(add_parameter_action::Error::InvalidParameter(
                add_parameter_action::ParameterValidationError::EmptyQuantityUnit,
            )) => GraphQLError::new("単位を入力してください", "VALIDATION_ERROR"),
            add_parameter::Error::Domain(add_parameter_action::Error::InvalidParameter(
                add_parameter_action::ParameterValidationError::NonPositiveQuantityAmount,
            )) => GraphQLError::new("数値は0より大きい値を入力してください", "VALIDATION_ERROR"),
            add_parameter::Error::Infrastructure(e) => internal_error(e),
        }
    }
}

impl From<add_parameter::Error> for async_graphql::Error {
    fn from(e: add_parameter::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for remove_parameter::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            remove_parameter::Error::NotFound => trial_not_found(),
            remove_parameter::Error::Domain(
                remove_parameter_action::Error::TrialAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのTrialのStepからParameterを削除できません",
                "VALIDATION_ERROR",
            ),
            remove_parameter::Error::Domain(remove_parameter_action::Error::StepNotFound) => {
                step_not_found()
            }
            remove_parameter::Error::Domain(
                remove_parameter_action::Error::StepAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのStepからParameterを削除できません",
                "VALIDATION_ERROR",
            ),
            remove_parameter::Error::Domain(remove_parameter_action::Error::ParameterNotFound) => {
                parameter_not_found()
            }
            remove_parameter::Error::Infrastructure(e) => internal_error(e),
        }
    }
}

impl From<remove_parameter::Error> for async_graphql::Error {
    fn from(e: remove_parameter::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for update_parameter::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            update_parameter::Error::NotFound => trial_not_found(),
            update_parameter::Error::Domain(
                update_parameter_action::Error::TrialAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのTrialのParameterは更新できません",
                "VALIDATION_ERROR",
            ),
            update_parameter::Error::Domain(update_parameter_action::Error::StepNotFound) => {
                step_not_found()
            }
            update_parameter::Error::Domain(
                update_parameter_action::Error::StepAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのStepのParameterは更新できません",
                "VALIDATION_ERROR",
            ),
            update_parameter::Error::Domain(update_parameter_action::Error::ParameterNotFound) => {
                parameter_not_found()
            }
            update_parameter::Error::Domain(
                update_parameter_action::Error::ParameterContentTypeMismatch,
            ) => GraphQLError::new("Parameterの種類は変更できません", "VALIDATION_ERROR"),
            update_parameter::Error::Domain(update_parameter_action::Error::InvalidParameter(
                update_parameter_action::ParameterValidationError::NegativeDurationValue,
            )) => GraphQLError::new("時間は0以上で入力してください", "VALIDATION_ERROR"),
            update_parameter::Error::Domain(update_parameter_action::Error::InvalidParameter(
                update_parameter_action::ParameterValidationError::EmptyQuantityUnit,
            )) => GraphQLError::new("単位を入力してください", "VALIDATION_ERROR"),
            update_parameter::Error::Domain(update_parameter_action::Error::InvalidParameter(
                update_parameter_action::ParameterValidationError::NonPositiveQuantityAmount,
            )) => GraphQLError::new("数値は0より大きい値を入力してください", "VALIDATION_ERROR"),
            update_parameter::Error::Infrastructure(e) => internal_error(e),
        }
    }
}

impl From<update_parameter::Error> for async_graphql::Error {
    fn from(e: update_parameter::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for complete_step::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            complete_step::Error::NotFound => trial_not_found(),
            complete_step::Error::Domain(complete_step_action::Error::TrialAlreadyCompleted) => {
                GraphQLError::new("完了済みのTrialのStepは完了できません", "VALIDATION_ERROR")
            }
            complete_step::Error::Domain(complete_step_action::Error::StepNotFound) => {
                step_not_found()
            }
            complete_step::Error::Domain(complete_step_action::Error::StepAlreadyCompleted) => {
                GraphQLError::new("Stepは既に完了しています", "VALIDATION_ERROR")
            }
            complete_step::Error::Infrastructure(e) => internal_error(e),
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
            get_trial::Error::Infrastructure(e) => internal_error(e),
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
            list_trials_by_project::Error::Infrastructure(e) => internal_error(e),
        }
    }
}

impl From<list_trials_by_project::Error> for async_graphql::Error {
    fn from(e: list_trials_by_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}
