use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::trial_status_validator;

pub use trial_status_validator::Error;

pub struct Command {}

pub fn validate(state: &Trial, _command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    Ok(())
}

pub fn execute(mut state: Trial, _command: Command) -> Trial {
    state.complete();
    state
}

pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;

    #[test]
    fn test_complete_trial_success() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let result = run(trial, Command {});
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_returns_error_when_already_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete();

        let result = run(trial, Command {});
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }
}
