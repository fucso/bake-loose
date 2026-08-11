//! Step に紐づく既存パラメーターの内容を更新するアクション
//!
//! ParameterContent の種類（バリアント）は変更不可。value/amount/unit などの
//! 末端の値のみを変更できる。種類を変えたい場合は remove_parameter + add_parameter を使う。

use crate::domain::models::parameter::{ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    parameter_validator, parameter_variant_validator, step_existence_validator,
    step_status_validator, trial_status_validator,
};

pub use parameter_validator::Error as ParameterValidationError;

pub struct Command {
    pub step_id: StepId,
    pub parameter_id: ParameterId,
    pub content: ParameterContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    ParameterNotFound,
    ParameterContentTypeMismatch,
    InvalidParameter(ParameterValidationError),
}

impl From<trial_status_validator::Error> for Error {
    fn from(_: trial_status_validator::Error) -> Self {
        Error::TrialAlreadyCompleted
    }
}

impl From<step_existence_validator::Error> for Error {
    fn from(_: step_existence_validator::Error) -> Self {
        Error::StepNotFound
    }
}

impl From<step_status_validator::Error> for Error {
    fn from(_: step_status_validator::Error) -> Self {
        Error::StepAlreadyCompleted
    }
}

impl From<parameter_variant_validator::Error> for Error {
    fn from(_: parameter_variant_validator::Error) -> Self {
        Error::ParameterContentTypeMismatch
    }
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    step_existence_validator::require_exists(state, &command.step_id)?;
    let step = state
        .step(&command.step_id)
        .expect("step existence already validated");
    step_status_validator::require_in_progress(step)?;

    let parameter = step
        .parameter(&command.parameter_id)
        .ok_or(Error::ParameterNotFound)?;
    parameter_variant_validator::require_same_variant(parameter.content(), &command.content)?;
    parameter_validator::validate(&command.content).map_err(Error::InvalidParameter)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &command.step_id)
        .expect("validated to exist");
    let parameter = step
        .parameters_mut()
        .iter_mut()
        .find(|p| p.id() == &command.parameter_id)
        .expect("validated to exist");
    parameter.set_content(command.content);

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
    use crate::domain::models::parameter::{
        DurationUnit, DurationValue, Parameter, ParameterValue,
    };
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step_and_parameter(content: ParameterContent) -> (Trial, StepId, ParameterId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        let parameter = Parameter::new(step_id.clone(), content);
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        trial.add_step(step);
        (trial, step_id, parameter_id)
    }

    #[test]
    fn test_update_parameter_replaces_content() {
        let (trial, step_id, parameter_id) =
            trial_with_step_and_parameter(ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            });
        let command = Command {
            step_id: step_id.clone(),
            parameter_id: parameter_id.clone(),
            content: ParameterContent::Text {
                value: "打ち粉を多めに".to_string(),
            },
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(
            step.parameters()[0].content(),
            &ParameterContent::Text {
                value: "打ち粉を多めに".to_string(),
            }
        );
        assert_eq!(step.parameters()[0].id(), &parameter_id);
    }

    #[test]
    fn test_update_parameter_replaces_key_value_amount() {
        let (trial, step_id, parameter_id) =
            trial_with_step_and_parameter(ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            });
        let command = Command {
            step_id: step_id.clone(),
            parameter_id,
            content: ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 350.0,
                    unit: "g".to_string(),
                },
            },
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(
            step.parameters()[0].content(),
            &ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 350.0,
                    unit: "g".to_string(),
                },
            }
        );
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id, parameter_id) =
            trial_with_step_and_parameter(ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            });
        trial.complete();
        let command = Command {
            step_id,
            parameter_id,
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        assert_eq!(run(trial, command), Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _, parameter_id) = trial_with_step_and_parameter(ParameterContent::Text {
            value: "打ち粉を追加".to_string(),
        });
        let command = Command {
            step_id: StepId::new(),
            parameter_id,
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        assert_eq!(run(trial, command), Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut completed_step = Step::from_raw(
            StepId::new(),
            trial.id().clone(),
            "こね".to_string(),
            0,
            Some(crate::domain::timezone::JstDateTime::now()),
            Some(crate::domain::timezone::JstDateTime::now()),
            Vec::new(),
        );
        let step_id = completed_step.id().clone();
        let parameter = Parameter::new(
            step_id.clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        completed_step.add_parameter(parameter);
        trial.add_step(completed_step);

        let command = Command {
            step_id,
            parameter_id,
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        assert_eq!(run(trial, command), Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_parameter_not_found() {
        let (trial, step_id, _) = trial_with_step_and_parameter(ParameterContent::Text {
            value: "打ち粉を追加".to_string(),
        });
        let command = Command {
            step_id,
            parameter_id: ParameterId::new(),
            content: ParameterContent::Text {
                value: "更新".to_string(),
            },
        };

        assert_eq!(run(trial, command), Err(Error::ParameterNotFound));
    }

    #[test]
    fn test_returns_error_when_content_type_differs() {
        let (trial, step_id, parameter_id) =
            trial_with_step_and_parameter(ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            });
        let command = Command {
            step_id,
            parameter_id,
            content: ParameterContent::Duration {
                duration: DurationValue::new(90.0, DurationUnit::Minute),
                note: "一次発酵".to_string(),
            },
        };

        assert_eq!(
            run(trial, command),
            Err(Error::ParameterContentTypeMismatch)
        );
    }

    #[test]
    fn test_returns_error_when_new_content_invalid() {
        let (trial, step_id, parameter_id) =
            trial_with_step_and_parameter(ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            });
        let command = Command {
            step_id,
            parameter_id,
            content: ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "   ".to_string(),
                },
            },
        };

        assert_eq!(
            run(trial, command),
            Err(Error::InvalidParameter(
                ParameterValidationError::EmptyQuantityUnit
            ))
        );
    }
}
