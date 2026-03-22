//! add_step アクション：既存の Trial に新しい Step を追加する

use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterContent};
use crate::domain::models::step::Step;
use crate::domain::models::trial::{Trial, TrialStatus};

pub struct ParameterInput {
    pub content: ParameterContent,
}

pub struct Command {
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValidationError {
    EmptyKey,
    EmptyNote,
    EmptyText,
    EmptyDurationNote,
    InvalidDurationValue,
}

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
    if state.status() == &TrialStatus::Completed {
        return Err(Error::TrialAlreadyCompleted);
    }
    if command.name.trim().is_empty() {
        return Err(Error::EmptyStepName);
    }
    for (i, param) in command.parameters.iter().enumerate() {
        validate_parameter_content(&param.content).map_err(|reason| Error::InvalidParameter {
            parameter_index: i,
            reason,
        })?;
    }
    Ok(())
}

fn validate_parameter_content(content: &ParameterContent) -> Result<(), ParameterValidationError> {
    match content {
        ParameterContent::KeyValue { key, .. } => {
            if key.trim().is_empty() {
                return Err(ParameterValidationError::EmptyKey);
            }
        }
        ParameterContent::Duration { duration, note } => {
            if duration.value < 0.0 {
                return Err(ParameterValidationError::InvalidDurationValue);
            }
            if note.trim().is_empty() {
                return Err(ParameterValidationError::EmptyDurationNote);
            }
        }
        ParameterContent::TimeMarker { at, note } => {
            if at.value < 0.0 {
                return Err(ParameterValidationError::InvalidDurationValue);
            }
            if note.trim().is_empty() {
                return Err(ParameterValidationError::EmptyNote);
            }
        }
        ParameterContent::Text { value } => {
            if value.trim().is_empty() {
                return Err(ParameterValidationError::EmptyText);
            }
        }
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
    use chrono::TimeZone;

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
    fn test_add_step_to_empty_trial() {
        let trial = in_progress_trial();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.steps().len(), 1);
        assert_eq!(result.steps()[0].name(), "捏ね");
    }

    #[test]
    fn test_add_step_position_is_zero_for_first() {
        let trial = in_progress_trial();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.steps()[0].position(), 0);
    }

    #[test]
    fn test_add_step_position_increments() {
        let trial = in_progress_trial();

        let command1 = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let trial = run(trial, command1).unwrap();

        let command2 = Command {
            name: "一次発酵".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let trial = run(trial, command2).unwrap();

        assert_eq!(trial.steps()[0].position(), 0);
        assert_eq!(trial.steps()[1].position(), 1);
    }

    #[test]
    fn test_add_step_with_parameters() {
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
        assert_eq!(result.steps()[0].parameters().len(), 1);
    }

    #[test]
    fn test_add_step_with_started_at() {
        let trial = in_progress_trial();
        let started = Utc::now();
        let command = Command {
            name: "捏ね".to_string(),
            started_at: Some(started),
            parameters: vec![],
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.steps()[0].started_at(), Some(&started));
    }

    #[test]
    fn test_trial_updated_at_is_changed() {
        let old_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let trial = Trial::from_raw(
            TrialId::new(),
            ProjectId::new(),
            None,
            None,
            TrialStatus::InProgress,
            vec![],
            old_time,
            old_time,
        );

        let command = Command {
            name: "捏ね".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command).unwrap();
        assert!(*result.updated_at() > old_time);
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

    #[test]
    fn test_returns_error_when_step_name_whitespace_only() {
        let trial = in_progress_trial();
        let command = Command {
            name: "   ".to_string(),
            started_at: None,
            parameters: vec![],
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::EmptyStepName));
    }
}
