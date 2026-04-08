//! update_step アクション
//!
//! 既存の Step の名前・開始日時・パラメーターを更新する。

use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    parameter_validator, step_existence_validator, step_name_validator, step_status_validator,
    trial_status_validator,
};

pub use parameter_validator::Error as ParameterValidationError;

/// 追加するパラメーターの入力
pub struct ParameterInput {
    pub content: ParameterContent,
}

/// コマンド
pub struct Command {
    /// 更新対象の Step ID
    pub step_id: StepId,
    /// 新しい名前（None の場合は変更なし）
    pub name: Option<String>,
    /// 新しい開始日時（None の場合は変更なし、Some(None) の場合はクリア）
    pub started_at: Option<Option<DateTime<Utc>>>,
    /// 追加するパラメーター
    pub add_parameters: Vec<ParameterInput>,
    /// 削除するパラメーター ID
    pub remove_parameter_ids: Vec<ParameterId>,
}

/// エラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Trial が既に完了している
    TrialAlreadyCompleted,
    /// 指定された Step が存在しない
    StepNotFound,
    /// Step が既に完了している
    StepAlreadyCompleted,
    /// 名前が空文字
    EmptyStepName,
    /// パラメーターが不正
    InvalidParameter {
        parameter_index: usize,
        reason: ParameterValidationError,
    },
    /// 削除対象のパラメーターが存在しない
    ParameterNotFound { parameter_id: ParameterId },
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    // 1. Trial のステータスチェック
    trial_status_validator::require_in_progress(state)
        .map_err(|_| Error::TrialAlreadyCompleted)?;

    // 2. Step の存在チェック
    let step = step_existence_validator::require_exists(state, &command.step_id)
        .map_err(|_| Error::StepNotFound)?;

    // 3. Step のステータスチェック
    step_status_validator::require_in_progress(step)
        .map_err(|_| Error::StepAlreadyCompleted)?;

    // 4. 名前が空文字でないかチェック
    step_name_validator::validate_optional(&command.name)
        .map_err(|_| Error::EmptyStepName)?;

    // 5. add_parameters の検証
    for (i, param_input) in command.add_parameters.iter().enumerate() {
        parameter_validator::validate_content(&param_input.content).map_err(|e| {
            Error::InvalidParameter {
                parameter_index: i,
                reason: e,
            }
        })?;
    }

    // 6. remove_parameter_ids の存在チェック
    for param_id in &command.remove_parameter_ids {
        if !step.parameters().iter().any(|p| p.id() == param_id) {
            return Err(Error::ParameterNotFound {
                parameter_id: param_id.clone(),
            });
        }
    }

    Ok(())
}

/// 状態遷移（validate 成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    if let Some(step) = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &command.step_id)
    {
        // 名前の更新
        if let Some(name) = command.name {
            step.set_name(name);
        }

        // 開始日時の更新
        if let Some(started_at) = command.started_at {
            step.set_started_at(started_at);
        }

        // パラメーターの削除
        for param_id in &command.remove_parameter_ids {
            step.remove_parameter(param_id);
        }

        // パラメーターの追加
        for param_input in command.add_parameters {
            let parameter = Parameter::new(step.id().clone(), param_input.content);
            step.add_parameter(parameter);
        }
    }

    state.touch();
    state
}

/// validate + execute
pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{DurationUnit, DurationValue, ParameterContent};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::Trial;

    fn make_trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), Some("テスト試行".to_string()), None);
        let step = Step::new(trial.id().clone(), "準備".to_string(), 1);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    fn make_trial_with_step_and_parameter() -> (Trial, StepId, ParameterId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "準備".to_string(), 1);
        let step_id = step.id().clone();
        let param = Parameter::new(
            step_id.clone(),
            ParameterContent::Text {
                value: "テキスト".to_string(),
            },
        );
        let param_id = param.id().clone();
        step.add_parameter(param);
        trial.add_step(step);
        (trial, step_id, param_id)
    }

    // --- 正常系 ---

    #[test]
    fn test_update_step_success() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            name: Some("新しいステップ名".to_string()),
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.name(), "新しいステップ名");
    }

    #[test]
    fn test_add_and_remove_parameters() {
        let (trial, step_id, param_id) = make_trial_with_step_and_parameter();
        let command = Command {
            step_id: step_id.clone(),
            name: None,
            started_at: None,
            add_parameters: vec![ParameterInput {
                content: ParameterContent::Text {
                    value: "新しいメモ".to_string(),
                },
            }],
            remove_parameter_ids: vec![param_id],
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
    }

    // --- 異常系 ---

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id) = make_trial_with_step();
        trial.complete();

        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _step_id) = make_trial_with_step();
        let command = Command {
            step_id: StepId::new(),
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "完了ステップ".to_string(), 1);
        let step_id = step.id().clone();
        step.complete(Utc::now());
        trial.add_step(step);

        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_name_empty() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id,
            name: Some("".to_string()),
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::EmptyStepName));
    }

    #[test]
    fn test_none_name_does_not_trigger_validation() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            name: None, // None の場合はバリデーションをスキップ
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert!(result.is_ok());
    }

    #[test]
    fn test_returns_error_when_parameter_invalid() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![ParameterInput {
                content: ParameterContent::Text {
                    value: "".to_string(),
                },
            }],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::InvalidParameter {
                parameter_index: 0,
                reason: ParameterValidationError::EmptyText,
            })
        );
    }

    #[test]
    fn test_validates_duration_parameter() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![ParameterInput {
                content: ParameterContent::Duration {
                    duration: DurationValue::new(-1.0, DurationUnit::Minute),
                    note: "発酵".to_string(),
                },
            }],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::InvalidParameter {
                parameter_index: 0,
                reason: ParameterValidationError::InvalidDurationValue,
            })
        );
    }

    #[test]
    fn test_validates_time_marker_parameter() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![ParameterInput {
                content: ParameterContent::TimeMarker {
                    at: DurationValue::new(30.0, DurationUnit::Minute),
                    note: "".to_string(), // 空のnote
                },
            }],
            remove_parameter_ids: vec![],
        };
        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::InvalidParameter {
                parameter_index: 0,
                reason: ParameterValidationError::EmptyNote,
            })
        );
    }

    #[test]
    fn test_returns_error_when_parameter_not_found() {
        let (trial, step_id) = make_trial_with_step();
        let non_existent_param_id = ParameterId::new();
        let command = Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: vec![],
            remove_parameter_ids: vec![non_existent_param_id.clone()],
        };
        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::ParameterNotFound {
                parameter_id: non_existent_param_id,
            })
        );
    }
}
