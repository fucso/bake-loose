use crate::domain::models::trial::Trial;
use crate::domain::timezone::JstDateTime;
use crate::domain::validators::trial::trial_status_validator;

pub use trial_status_validator::Error;

pub struct Command {
    pub completed_at: Option<JstDateTime>,
}

pub fn validate(state: &Trial, _command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
///
/// completed_at が未指定の場合は現在時刻を採用する（[`Trial::complete`] に委譲）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    state.complete(command.completed_at);
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
        let result = run(trial, Command { completed_at: None });
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_complete_trial_uses_specified_completed_at() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let completed_at = JstDateTime::now();

        let result = run(
            trial,
            Command {
                completed_at: Some(completed_at),
            },
        );

        assert_eq!(result.unwrap().completed_at(), Some(&completed_at));
    }

    #[test]
    fn test_complete_trial_defaults_completed_at_to_now_when_unspecified() {
        let trial = Trial::new(ProjectId::new(), None, None);

        let result = run(trial, Command { completed_at: None });

        assert!(result.unwrap().completed_at().is_some());
    }

    #[test]
    fn test_returns_error_when_already_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete(None);

        let result = run(trial, Command { completed_at: None });
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }
}
