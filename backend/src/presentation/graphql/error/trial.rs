//! Trial 関連のユースケースエラー -> GraphQL エラー変換

use async_graphql::ErrorExtensions;

use crate::domain::actions::trial::{
    add_parameter as add_parameter_action, add_step as add_step_action,
    complete_step as complete_step_action, complete_trial as complete_trial_action,
    create_trial as create_trial_action, remove_parameter as remove_parameter_action,
    update_parameter as update_parameter_action, update_step as update_step_action,
    update_trial as update_trial_action,
};
use crate::presentation::graphql::error::common::{conflict_error, GraphQLError, UserFacingError};
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

/// 「指定されたProjectが見つかりません」エラー
fn project_not_found() -> GraphQLError {
    GraphQLError::new("指定されたProjectが見つかりません", "NOT_FOUND")
}

/// Action が実際には返し得ない domain error variant を受け取った場合の内部エラー
///
/// trial ドメインのエラーは `trial_error::Error` に一元管理されており、各 Action の
/// `Error` は同じ型を再エクスポートしたものにすぎない。そのため個々の Action が実際に
/// 返しうる variant を全て網羅しても、型としては他 Action 用の variant も理論上存在する。
/// ここに来ることは実装上あり得ないが、`match` を網羅させるためのフォールバックとして残す。
fn unexpected_domain_error(
    action: &str,
    error: &crate::domain::errors::trial_error::Error,
) -> GraphQLError {
    log::error!(
        "Unexpected trial domain error variant for {}: {:?}",
        action,
        error
    );
    GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
}

/// 外部キー違反で見つからなかった参照先を、エンティティ名に応じたメッセージへ振り分ける
///
/// 参照先は Trial とは限らず、例えば `parameters_step_id_fkey` の違反なら Step が
/// 並行して削除されている。一律に Trial のメッセージを返すと、
/// 実際に消えたのは Step なのに「Trialが見つかりません」と誤って案内してしまう。
///
/// 全ユースケースで同じ振り分けを繰り返さないよう、ここに集約する。
/// 未知のエンティティ名は内部名を露出させないため汎用メッセージへ倒す。
fn reference_not_found(entity: &str) -> GraphQLError {
    match entity {
        "trial" => trial_not_found(),
        "step" => step_not_found(),
        "parameter" => parameter_not_found(),
        "project" => project_not_found(),
        _ => GraphQLError::new("指定されたデータが見つかりません", "NOT_FOUND"),
    }
}

impl UserFacingError for create_trial::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_trial::Error::ProjectNotFound => project_not_found(),
            create_trial::Error::Domain(create_trial_action::Error::EmptyTrialName) => {
                GraphQLError::new("Trial名を入力してください", "VALIDATION_ERROR")
            }
            create_trial::Error::Domain(create_trial_action::Error::TrialNameTooLong {
                max,
                ..
            }) => GraphQLError::new(
                format!("Trial名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
            create_trial::Error::Domain(other) => unexpected_domain_error("create_trial", other),
            create_trial::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            update_trial::Error::Domain(update_trial_action::Error::EmptyTrialName) => {
                GraphQLError::new("Trial名を入力してください", "VALIDATION_ERROR")
            }
            update_trial::Error::Domain(update_trial_action::Error::TrialNameTooLong {
                max,
                ..
            }) => GraphQLError::new(
                format!("Trial名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
            update_trial::Error::Domain(other) => unexpected_domain_error("update_trial", other),
            update_trial::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            update_trial::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            complete_trial::Error::Domain(other) => {
                unexpected_domain_error("complete_trial", other)
            }
            complete_trial::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            complete_trial::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            add_step::Error::Domain(add_step_action::Error::EmptyStepName) => {
                GraphQLError::new("Step名を入力してください", "VALIDATION_ERROR")
            }
            add_step::Error::Domain(add_step_action::Error::StepNameTooLong { max, .. }) => {
                GraphQLError::new(
                    format!("Step名は{}文字以内で入力してください", max),
                    "VALIDATION_ERROR",
                )
            }
            add_step::Error::Domain(other) => unexpected_domain_error("add_step", other),
            add_step::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            add_step::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            update_step::Error::Domain(update_step_action::Error::EmptyStepName) => {
                GraphQLError::new("Step名を入力してください", "VALIDATION_ERROR")
            }
            update_step::Error::Domain(update_step_action::Error::StepNameTooLong {
                max, ..
            }) => GraphQLError::new(
                format!("Step名は{}文字以内で入力してください", max),
                "VALIDATION_ERROR",
            ),
            update_step::Error::Domain(other) => unexpected_domain_error("update_step", other),
            update_step::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            update_step::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            add_parameter::Error::Domain(add_parameter_action::Error::NegativeDurationValue) => {
                GraphQLError::new("時間は0以上で入力してください", "VALIDATION_ERROR")
            }
            add_parameter::Error::Domain(add_parameter_action::Error::EmptyQuantityUnit) => {
                GraphQLError::new("単位を入力してください", "VALIDATION_ERROR")
            }
            add_parameter::Error::Domain(
                add_parameter_action::Error::NonPositiveQuantityAmount,
            ) => GraphQLError::new("数値は0より大きい値を入力してください", "VALIDATION_ERROR"),
            add_parameter::Error::Domain(other) => unexpected_domain_error("add_parameter", other),
            add_parameter::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            add_parameter::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            remove_parameter::Error::Domain(other) => {
                unexpected_domain_error("remove_parameter", other)
            }
            remove_parameter::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            remove_parameter::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            update_parameter::Error::Domain(
                update_parameter_action::Error::NegativeDurationValue,
            ) => GraphQLError::new("時間は0以上で入力してください", "VALIDATION_ERROR"),
            update_parameter::Error::Domain(update_parameter_action::Error::EmptyQuantityUnit) => {
                GraphQLError::new("単位を入力してください", "VALIDATION_ERROR")
            }
            update_parameter::Error::Domain(
                update_parameter_action::Error::NonPositiveQuantityAmount,
            ) => GraphQLError::new("数値は0より大きい値を入力してください", "VALIDATION_ERROR"),
            update_parameter::Error::Domain(other) => {
                unexpected_domain_error("update_parameter", other)
            }
            update_parameter::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            update_parameter::Error::Conflict { entity, field } => conflict_error(entity, field),
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
            complete_step::Error::Domain(other) => unexpected_domain_error("complete_step", other),
            complete_step::Error::ReferenceNotFound { entity } => reference_not_found(entity),
            complete_step::Error::Conflict { entity, field } => conflict_error(entity, field),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn conflict(entity: &str, field: &str) -> (String, String) {
        (entity.to_string(), field.to_string())
    }

    /// 競合エラーは CONFLICT コードでユーザーにリトライを促す
    ///
    /// 制約の詳細（entity / field）はユーザー向けメッセージには出さず、ログにのみ残す。
    #[test]
    fn test_conflict_maps_to_conflict_code() {
        let expected =
            GraphQLError::new("他の操作と競合しました。もう一度お試しください", "CONFLICT");

        let (entity, field) = conflict("step", "trial_id_position");

        assert_eq!(
            add_step::Error::Conflict {
                entity: entity.clone(),
                field: field.clone()
            }
            .to_user_facing(),
            expected
        );
        assert_eq!(
            create_trial::Error::Conflict {
                entity: entity.clone(),
                field: field.clone()
            }
            .to_user_facing(),
            expected
        );
        assert_eq!(
            update_step::Error::Conflict {
                entity: entity.clone(),
                field: field.clone()
            }
            .to_user_facing(),
            expected
        );
        assert_eq!(
            add_parameter::Error::Conflict { entity, field }.to_user_facing(),
            expected
        );
    }

    /// 外部キー違反で消えたのが Step なら Step のメッセージを返す
    ///
    /// 以前は RepositoryError::NotFound のエンティティ名を捨てて一律
    /// `trial_not_found()` を返していたため、`parameters_step_id_fkey` 違反でも
    /// 「指定されたTrialが見つかりません」と誤って案内していた。
    #[test]
    fn test_reference_not_found_for_step_returns_step_message() {
        let error = add_parameter::Error::ReferenceNotFound {
            entity: "step".to_string(),
        }
        .to_user_facing();

        assert_eq!(
            error,
            GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
        );
    }

    /// 参照先のエンティティ名ごとに対応するメッセージへ振り分ける
    #[test]
    fn test_reference_not_found_dispatches_by_entity() {
        assert_eq!(
            reference_not_found("trial"),
            GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
        );
        assert_eq!(
            reference_not_found("step"),
            GraphQLError::new("指定されたStepが見つかりません", "NOT_FOUND")
        );
        assert_eq!(
            reference_not_found("parameter"),
            GraphQLError::new("指定されたParameterが見つかりません", "NOT_FOUND")
        );
        assert_eq!(
            reference_not_found("project"),
            GraphQLError::new("指定されたProjectが見つかりません", "NOT_FOUND")
        );
    }

    /// 未知のエンティティ名は内部名を露出させず汎用メッセージに倒す
    #[test]
    fn test_reference_not_found_falls_back_to_generic_message() {
        assert_eq!(
            reference_not_found("feedback"),
            GraphQLError::new("指定されたデータが見つかりません", "NOT_FOUND")
        );
    }

    /// 集約ルートが見つからない NotFound は従来どおり Trial のメッセージ
    #[test]
    fn test_not_found_still_maps_to_trial_message() {
        assert_eq!(
            add_parameter::Error::NotFound.to_user_facing(),
            GraphQLError::new("指定されたTrialが見つかりません", "NOT_FOUND")
        );
    }

    /// Infrastructure エラーは従来どおり INTERNAL_ERROR のまま
    #[test]
    fn test_infrastructure_still_maps_to_internal_error() {
        assert_eq!(
            add_step::Error::Infrastructure("boom".to_string()).to_user_facing(),
            GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
        );
    }
}
