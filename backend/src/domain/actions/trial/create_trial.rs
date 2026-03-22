use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterValue};
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;
use crate::domain::models::trial::Trial;

pub struct ParameterInput {
    pub content: ParameterContent,
}

pub struct StepInput {
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Vec<ParameterInput>,
}

pub struct Command {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub steps: Vec<StepInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValidationError {
    EmptyKey,
    EmptyTextValue,
    EmptyTimeMarkerNote,
    EmptyDurationNote,
    InvalidQuantity,
    InvalidDurationValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyStepName {
        step_index: usize,
    },
    InvalidParameter {
        step_index: usize,
        parameter_index: usize,
        reason: ParameterValidationError,
    },
}

pub fn validate(command: &Command) -> Result<(), Error> {
    for (step_index, step) in command.steps.iter().enumerate() {
        if step.name.trim().is_empty() {
            return Err(Error::EmptyStepName { step_index });
        }
        for (parameter_index, param) in step.parameters.iter().enumerate() {
            validate_parameter_content(&param.content, step_index, parameter_index)?;
        }
    }
    Ok(())
}

fn validate_parameter_content(
    content: &ParameterContent,
    step_index: usize,
    parameter_index: usize,
) -> Result<(), Error> {
    match content {
        ParameterContent::KeyValue { key, value } => {
            if key.trim().is_empty() {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::EmptyKey,
                });
            }
            match value {
                ParameterValue::Text { value: text } if text.trim().is_empty() => {
                    return Err(Error::InvalidParameter {
                        step_index,
                        parameter_index,
                        reason: ParameterValidationError::EmptyTextValue,
                    });
                }
                ParameterValue::Quantity { amount, .. } if *amount < 0.0 => {
                    return Err(Error::InvalidParameter {
                        step_index,
                        parameter_index,
                        reason: ParameterValidationError::InvalidQuantity,
                    });
                }
                _ => {}
            }
        }
        ParameterContent::TimeMarker { at, note } => {
            if at.value < 0.0 {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::InvalidDurationValue,
                });
            }
            if note.trim().is_empty() {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::EmptyTimeMarkerNote,
                });
            }
        }
        ParameterContent::Text { value } => {
            if value.trim().is_empty() {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::EmptyTextValue,
                });
            }
        }
        ParameterContent::Duration { duration, note } => {
            if duration.value < 0.0 {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::InvalidDurationValue,
                });
            }
            if note.trim().is_empty() {
                return Err(Error::InvalidParameter {
                    step_index,
                    parameter_index,
                    reason: ParameterValidationError::EmptyDurationNote,
                });
            }
        }
    }
    Ok(())
}

pub fn execute(command: Command) -> Trial {
    let mut trial = Trial::new(command.project_id, command.name, command.memo);

    for step_input in command.steps {
        let position = trial.next_step_position();
        let mut step = Step::new(trial.id().clone(), step_input.name, position);

        if let Some(started_at) = step_input.started_at {
            step.start(started_at);
        }

        for param_input in step_input.parameters {
            let parameter = Parameter::new(step.id().clone(), param_input.content);
            step.add_parameter(parameter);
        }

        trial.add_step(step);
    }

    trial
}

pub fn run(command: Command) -> Result<Trial, Error> {
    validate(&command)?;
    Ok(execute(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{
        DurationUnit, DurationValue, ParameterContent, ParameterValue,
    };
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;

    fn make_project_id() -> ProjectId {
        ProjectId::new()
    }

    // --- 正常系 ---

    #[test]
    fn test_create_trial_with_no_steps() {
        let command = Command {
            project_id: make_project_id(),
            name: Some("バゲット第1回".to_string()),
            memo: None,
            steps: vec![],
        };
        let trial = run(command).unwrap();
        assert!(trial.steps().is_empty());
    }

    #[test]
    fn test_create_trial_with_single_step() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "捏ね".to_string(),
                started_at: None,
                parameters: vec![],
            }],
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.steps().len(), 1);
        assert_eq!(trial.steps()[0].name(), "捏ね");
    }

    #[test]
    fn test_create_trial_with_multiple_steps() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![
                StepInput {
                    name: "捏ね".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
                StepInput {
                    name: "一次発酵".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
                StepInput {
                    name: "焼成".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
            ],
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.steps().len(), 3);
    }

    #[test]
    fn test_step_positions_are_sequential() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![
                StepInput {
                    name: "捏ね".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
                StepInput {
                    name: "一次発酵".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
                StepInput {
                    name: "焼成".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
            ],
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.steps()[0].position(), 0);
        assert_eq!(trial.steps()[1].position(), 1);
        assert_eq!(trial.steps()[2].position(), 2);
    }

    #[test]
    fn test_create_trial_with_parameters() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "捏ね".to_string(),
                started_at: None,
                parameters: vec![
                    ParameterInput {
                        content: ParameterContent::KeyValue {
                            key: "強力粉".to_string(),
                            value: ParameterValue::Quantity {
                                amount: 300.0,
                                unit: "g".to_string(),
                            },
                        },
                    },
                    ParameterInput {
                        content: ParameterContent::Duration {
                            duration: DurationValue::new(10.0, DurationUnit::Minute),
                            note: "捏ね時間".to_string(),
                        },
                    },
                ],
            }],
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.steps()[0].parameters().len(), 2);
    }

    #[test]
    fn test_trial_status_is_in_progress() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![],
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.status(), &TrialStatus::InProgress);
    }

    // --- 異常系 ---

    #[test]
    fn test_returns_error_when_step_name_empty() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "".to_string(),
                started_at: None,
                parameters: vec![],
            }],
        };
        let result = run(command);
        assert!(matches!(
            result,
            Err(Error::EmptyStepName { step_index: 0 })
        ));
    }

    #[test]
    fn test_error_contains_step_index() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![
                StepInput {
                    name: "捏ね".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
                StepInput {
                    name: "".to_string(),
                    started_at: None,
                    parameters: vec![],
                },
            ],
        };
        let result = run(command);
        assert!(matches!(
            result,
            Err(Error::EmptyStepName { step_index: 1 })
        ));
    }

    #[test]
    fn test_returns_error_when_key_value_key_empty() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "捏ね".to_string(),
                started_at: None,
                parameters: vec![ParameterInput {
                    content: ParameterContent::KeyValue {
                        key: "".to_string(),
                        value: ParameterValue::Text {
                            value: "300g".to_string(),
                        },
                    },
                }],
            }],
        };
        let result = run(command);
        assert!(matches!(
            result,
            Err(Error::InvalidParameter {
                step_index: 0,
                parameter_index: 0,
                reason: ParameterValidationError::EmptyKey,
            })
        ));
    }

    #[test]
    fn test_returns_error_when_time_marker_note_empty() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "焼成".to_string(),
                started_at: None,
                parameters: vec![ParameterInput {
                    content: ParameterContent::TimeMarker {
                        at: DurationValue::new(30.0, DurationUnit::Minute),
                        note: "".to_string(),
                    },
                }],
            }],
        };
        let result = run(command);
        assert!(matches!(
            result,
            Err(Error::InvalidParameter {
                step_index: 0,
                parameter_index: 0,
                reason: ParameterValidationError::EmptyTimeMarkerNote,
            })
        ));
    }

    #[test]
    fn test_error_contains_parameter_index() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
            steps: vec![StepInput {
                name: "焼成".to_string(),
                started_at: None,
                parameters: vec![
                    ParameterInput {
                        content: ParameterContent::Duration {
                            duration: DurationValue::new(10.0, DurationUnit::Minute),
                            note: "焼き時間".to_string(),
                        },
                    },
                    ParameterInput {
                        content: ParameterContent::TimeMarker {
                            at: DurationValue::new(30.0, DurationUnit::Minute),
                            note: "".to_string(),
                        },
                    },
                ],
            }],
        };
        let result = run(command);
        assert!(matches!(
            result,
            Err(Error::InvalidParameter {
                step_index: 0,
                parameter_index: 1,
                reason: ParameterValidationError::EmptyTimeMarkerNote,
            })
        ));
    }
}
