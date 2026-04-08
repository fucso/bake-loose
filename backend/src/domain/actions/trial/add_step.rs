//! add_step アクション：既存の Trial に新しい Step を追加する

use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterContent};
use crate::domain::models::step::Step;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    parameter_validator, step_name_validator, trial_status_validator,
};

pub struct ParameterInput {
    pub content: ParameterContent,
}

pub struct Command {
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterInput>,
}

pub use parameter_validator::Error as ParameterValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    EmptyStepName,
    InvalidParameter {
        parameter_index: usize,
        reason: ParameterValidationError,
    },
}

pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)
        .map_err(|_| Error::TrialAlreadyCompleted)?;
    step_name_validator::require_not_empty(&command.name)
        .map_err(|_| Error::EmptyStepName)?;

    for (i, param) in command.parameters.iter().enumerate() {
        parameter_validator::validate_content(&param.content).map_err(|e| {
            Error::InvalidParameter {
                parameter_index: i,
                reason: e,
            }
        })?;
    }
    Ok(())
}

pub fn execute(mut state: Trial, command: Command) -> Trial {
    let position = state.next_step_position();
    let mut step = Step::new(state.id().clone(), command.name, position);

    if let Some(started_at) = command.started_at {
        step.start(started_at);
    }

    for param in command.parameters {
        let parameter = Parameter::new(step.id().clone(), param.content);
        step.add_parameter(parameter);
    }

    state.add_step(step);
    state
}

pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{ParameterContent, ParameterValue};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::{Trial, TrialId, TrialStatus};

    fn in_progress_trial() -> Trial {
        Trial::new(ProjectId::new(), None, None)
    }

    fn completed_trial() -> Trial {
        let now = Utc::now();
        Trial::from_raw(
            TrialId::new(),
            ProjectId::new(),
            None,
            None,
            TrialStatus::Completed,
            vec![],
            now,
            now,
        )
    }

    #[test]
    fn test_add_step_success() {
        let trial = in_progress_trial();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![ParameterInput {
                content: ParameterContent::KeyValue {
                    key: "強力粉".to_string(),
                    value: ParameterValue::Quantity {
                        amount: 300.0,
                        unit: "g".to_string(),
                    },
                },
            }],
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.steps().len(), 1);
        assert_eq!(result.steps()[0].name(), "捏ね");
        assert_eq!(result.steps()[0].parameters().len(), 1);
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let trial = completed_trial();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_name_empty() {
        let trial = in_progress_trial();
        let command = Command {
            name: "".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::EmptyStepName));
    }

    #[test]
    fn test_returns_error_when_parameter_invalid() {
        let trial = in_progress_trial();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![ParameterInput {
                content: ParameterContent::KeyValue {
                    key: "".to_string(),
                    value: ParameterValue::Text {
                        value: "test".to_string(),
                    },
                },
            }],
        };
        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::InvalidParameter {
                parameter_index: 0,
                reason: ParameterValidationError::EmptyKey,
            })
        );
    }
}
