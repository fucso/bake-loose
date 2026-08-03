use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterId};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::timezone::JstDateTime;
use crate::domain::validators::trial::{
    parameter_validator, step_existence_validator, step_name_validator, step_status_validator,
    trial_status_validator,
};

pub struct ParameterInput {
    pub content: ParameterContent,
}

pub struct Command {
    pub step_id: StepId,
    /// Some の場合のみ変更
    pub name: Option<String>,
    /// None: 変更なし / Some(None): クリア / Some(Some(t)): t に設定
    pub started_at: Option<Option<JstDateTime>>,
    pub add_parameters: Vec<ParameterInput>,
    pub remove_parameter_ids: Vec<ParameterId>,
}

pub use parameter_validator::Error as ParameterValidationError;
pub use step_name_validator::Error as StepNameValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    InvalidStepName(StepNameValidationError),
    InvalidParameter {
        parameter_index: usize,
        reason: ParameterValidationError,
    },
    ParameterNotFound {
        parameter_id: ParameterId,
    },
}

pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state).map_err(|_| Error::TrialAlreadyCompleted)?;
    let step = step_existence_validator::require_exists(state, &command.step_id)
        .map_err(|_| Error::StepNotFound)?;
    step_status_validator::require_in_progress(step).map_err(|_| Error::StepAlreadyCompleted)?;

    if let Some(name) = &command.name {
        step_name_validator::validate(name).map_err(Error::InvalidStepName)?;
    }
    for (i, param) in command.add_parameters.iter().enumerate() {
        parameter_validator::validate(&param.content).map_err(|e| Error::InvalidParameter {
            parameter_index: i,
            reason: e,
        })?;
    }
    for parameter_id in &command.remove_parameter_ids {
        if !step.parameters().iter().any(|p| p.id() == parameter_id) {
            return Err(Error::ParameterNotFound {
                parameter_id: parameter_id.clone(),
            });
        }
    }
    Ok(())
}

pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step_id = command.step_id.clone();
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &step_id)
        .expect("validated to exist");

    if let Some(name) = command.name {
        step.set_name(name);
    }
    if let Some(started_at) = command.started_at {
        step.start(started_at);
    }
    for param in command.add_parameters {
        let parameter = Parameter::new(step.id().clone(), param.content);
        step.add_parameter(parameter);
    }
    for parameter_id in &command.remove_parameter_ids {
        step.remove_parameter(parameter_id);
    }

    state
}

pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::ParameterValue;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    fn base_command(step_id: StepId) -> Command {
        Command {
            step_id,
            name: None,
            started_at: None,
            add_parameters: Vec::new(),
            remove_parameter_ids: Vec::new(),
        }
    }

    #[test]
    fn test_update_step_name() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            name: Some("新名称".to_string()),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.name(), "新名称");
    }

    #[test]
    fn test_update_step_started_at_clear() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            started_at: Some(None),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.started_at(), None);
    }

    #[test]
    fn test_update_step_started_at_sets_value() {
        let (trial, step_id) = trial_with_step();
        let new_started_at =
            crate::domain::timezone::JstDateTime::now() - chrono::Duration::hours(1);
        let command = Command {
            started_at: Some(Some(new_started_at)),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.started_at(), Some(&new_started_at));
    }

    #[test]
    fn test_update_step_add_parameters() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            add_parameters: vec![ParameterInput {
                content: ParameterContent::Text {
                    value: "打ち粉を追加".to_string(),
                },
            }],
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
    }

    #[test]
    fn test_update_step_remove_parameters() {
        let (mut trial, step_id) = trial_with_step();
        let parameter = Parameter::new(
            step_id.clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        trial
            .steps_mut()
            .iter_mut()
            .find(|s| s.id() == &step_id)
            .unwrap()
            .add_parameter(parameter);

        let command = Command {
            remove_parameter_ids: vec![parameter_id],
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id) = trial_with_step();
        trial.complete();

        let result = run(trial, base_command(step_id));
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _) = trial_with_step();

        let result = run(trial, base_command(StepId::new()));
        assert_eq!(result, Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let completed_step = Step::from_raw(
            StepId::new(),
            trial.id().clone(),
            "こね".to_string(),
            0,
            Some(crate::domain::timezone::JstDateTime::now()),
            Some(crate::domain::timezone::JstDateTime::now()),
            Vec::new(),
        );
        let step_id = completed_step.id().clone();
        trial.add_step(completed_step);

        let result = run(trial, base_command(step_id));
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_new_name_empty() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            name: Some("".to_string()),
            ..base_command(step_id)
        };

        let result = run(trial, command);
        assert!(matches!(result, Err(Error::InvalidStepName(_))));
    }

    #[test]
    fn test_returns_error_when_parameter_invalid() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            add_parameters: vec![ParameterInput {
                content: ParameterContent::KeyValue {
                    key: "強力粉".to_string(),
                    value: ParameterValue::Quantity {
                        amount: 300.0,
                        unit: "   ".to_string(),
                    },
                },
            }],
            ..base_command(step_id)
        };

        let result = run(trial, command);
        assert!(matches!(
            result,
            Err(Error::InvalidParameter {
                parameter_index: 0,
                ..
            })
        ));
    }

    #[test]
    fn test_returns_error_when_remove_parameter_not_found() {
        let (trial, step_id) = trial_with_step();
        let missing_id = ParameterId::new();
        let command = Command {
            remove_parameter_ids: vec![missing_id.clone()],
            ..base_command(step_id)
        };

        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::ParameterNotFound {
                parameter_id: missing_id
            })
        );
    }
}
