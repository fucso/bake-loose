use chrono::{DateTime, Utc};

use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    step_existence_validator, step_status_validator, trial_status_validator,
};

pub struct Command {
    pub step_id: StepId,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state).map_err(|_| Error::TrialAlreadyCompleted)?;
    let step = step_existence_validator::require_exists(state, &command.step_id)
        .map_err(|_| Error::StepNotFound)?;
    step_status_validator::require_in_progress(step).map_err(|_| Error::StepAlreadyCompleted)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|step| step.id() == &command.step_id)
        .expect("step must exist (validated)");
    step.complete(command.completed_at);
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
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[test]
    fn test_run_completes_step_with_specified_completed_at() {
        let (trial, step_id) = trial_with_step();
        let completed_at = Utc::now();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: Some(completed_at),
        };

        let trial = run(trial, command).unwrap();

        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.completed_at(), Some(&completed_at));
    }

    #[test]
    fn test_run_defaults_completed_at_to_now_when_unspecified() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };

        let trial = run(trial, command).unwrap();

        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.completed_at().is_some());
    }

    #[test]
    fn test_run_err_when_trial_already_completed() {
        let (mut trial, step_id) = trial_with_step();
        trial.complete();
        let command = Command {
            step_id,
            completed_at: None,
        };

        assert_eq!(run(trial, command), Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_run_err_when_step_not_found() {
        let (trial, _) = trial_with_step();
        let command = Command {
            step_id: StepId::new(),
            completed_at: None,
        };

        assert_eq!(run(trial, command), Err(Error::StepNotFound));
    }

    #[test]
    fn test_run_err_when_step_already_completed() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };
        let trial = run(trial, command).unwrap();

        let command = Command {
            step_id,
            completed_at: None,
        };
        assert_eq!(run(trial, command), Err(Error::StepAlreadyCompleted));
    }
}
