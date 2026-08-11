use crate::domain::models::step::Step;
use crate::domain::models::trial::Trial;
use crate::domain::timezone::JstDateTime;
use crate::domain::validators::trial::{step_name_validator, trial_status_validator};

pub use step_name_validator::Error as StepNameError;

pub struct Command {
    pub name: String,
    pub started_at: Option<JstDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    InvalidStepName(StepNameError),
}

impl From<trial_status_validator::Error> for Error {
    fn from(_: trial_status_validator::Error) -> Self {
        Error::TrialAlreadyCompleted
    }
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    step_name_validator::validate(&command.name).map_err(Error::InvalidStepName)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
///
/// position は既存 Step 数から自動採番する。
/// パラメーターの追加は `add_parameter` アクションの責務であり、このアクションのスコープ外。
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let position = state.steps().len() as i16;
    let step = Step::new(
        state.id().clone(),
        command.name,
        position,
        command.started_at,
    );
    state.add_step(step);
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

    fn command(name: &str) -> Command {
        Command {
            name: name.to_string(),
            started_at: None,
        }
    }

    #[test]
    fn test_run_appends_step_with_position_zero_for_first_step() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial = run(trial, command("こね")).unwrap();

        assert_eq!(trial.steps().len(), 1);
        assert_eq!(trial.steps()[0].name(), "こね");
        assert_eq!(trial.steps()[0].position(), 0);
    }

    #[test]
    fn test_run_auto_increments_position_for_subsequent_steps() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial = run(trial, command("こね")).unwrap();
        let trial = run(trial, command("発酵")).unwrap();
        let trial = run(trial, command("焼成")).unwrap();

        let positions: Vec<i16> = trial.steps().iter().map(|s| s.position()).collect();
        assert_eq!(positions, vec![0, 1, 2]);
    }

    #[test]
    fn test_run_defaults_started_at_to_now_when_unspecified() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial = run(trial, command("こね")).unwrap();

        assert!(trial.steps()[0].started_at().is_some());
    }

    #[test]
    fn test_run_uses_specified_started_at() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let started_at = crate::domain::timezone::JstDateTime::now() - chrono::Duration::hours(1);
        let mut cmd = command("こね");
        cmd.started_at = Some(started_at);

        let trial = run(trial, cmd).unwrap();
        assert_eq!(trial.steps()[0].started_at(), Some(&started_at));
    }

    #[test]
    fn test_run_creates_step_without_parameters() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial = run(trial, command("こね")).unwrap();
        assert!(trial.steps()[0].parameters().is_empty());
    }

    #[test]
    fn test_run_err_when_trial_already_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete();

        assert_eq!(
            run(trial, command("こね")),
            Err(Error::TrialAlreadyCompleted)
        );
    }

    #[test]
    fn test_run_err_when_step_name_is_empty() {
        let trial = Trial::new(ProjectId::new(), None, None);

        assert_eq!(
            run(trial, command("")),
            Err(Error::InvalidStepName(StepNameError::EmptyName))
        );
    }

    #[test]
    fn test_run_err_when_step_name_too_long() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let too_long = "a".repeat(101);

        assert_eq!(
            run(trial, command(&too_long)),
            Err(Error::InvalidStepName(StepNameError::NameTooLong {
                max: 100,
                actual: 101,
            }))
        );
    }

    #[test]
    fn test_validate_does_not_mutate_state() {
        let trial = Trial::new(ProjectId::new(), None, None);
        assert!(validate(&trial, &command("こね")).is_ok());
        assert!(trial.steps().is_empty());
    }

    #[test]
    fn test_execute_preserves_trial_id_and_adds_step() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let trial_id = trial.id().clone();
        let updated = execute(trial, command("こね"));

        assert_eq!(updated.id(), &trial_id);
        assert_eq!(updated.steps().len(), 1);
    }
}
