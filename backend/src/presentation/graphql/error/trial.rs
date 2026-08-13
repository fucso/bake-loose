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
    add_step, complete_step, complete_trial, create_trial, get_trial, list_trials_by_project,
    update_parameter, update_step, update_trial,
};

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

impl UserFacingError for update_parameter::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            update_parameter::Error::NotFound => {
                GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
            }
            update_parameter::Error::Domain(
                update_parameter_action::Error::TrialAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのTrialのParameterは更新できません",
                "VALIDATION_ERROR",
            ),
            update_parameter::Error::Domain(update_parameter_action::Error::StepNotFound) => {
                GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
            }
            update_parameter::Error::Domain(
                update_parameter_action::Error::StepAlreadyCompleted,
            ) => GraphQLError::new(
                "完了済みのStepのParameterは更新できません",
                "VALIDATION_ERROR",
            ),
            update_parameter::Error::Domain(update_parameter_action::Error::ParameterNotFound) => {
                GraphQLError::new("指定されたParameterが見つかりません", "NOT_FOUND")
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
            update_parameter::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
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
