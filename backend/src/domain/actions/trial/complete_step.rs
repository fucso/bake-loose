use crate::domain::clock::{Clock, SystemClock};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::timezone::JstDateTime;
use crate::domain::validators::trial::{
    step_existence_validator, step_status_validator, trial_status_validator,
};

pub struct Command {
    pub step_id: StepId,
    pub completed_at: Option<JstDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
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

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    step_existence_validator::require_exists(state, &command.step_id)?;
    let step = state
        .steps()
        .iter()
        .find(|step| step.id() == &command.step_id)
        .expect("step existence already validated");
    step_status_validator::require_in_progress(step)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
///
/// completed_at が未指定の場合は clock から現在時刻を採用する
pub fn execute(mut state: Trial, command: Command, clock: &dyn Clock) -> Trial {
    let completed_at = command.completed_at.unwrap_or_else(|| clock.now());
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|step| step.id() == &command.step_id)
        .expect("step must exist (validated)");
    step.complete(Some(completed_at));
    state
}

/// validate + execute（現在時刻には [`SystemClock`] を使用する）
pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    run_with_clock(state, command, &SystemClock)
}

/// validate + execute（テスト等で Clock を差し替える場合に使用する）
pub fn run_with_clock(state: Trial, command: Command, clock: &dyn Clock) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command, clock))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    /// テスト用に固定の時刻を返す Clock
    struct FixedClock(JstDateTime);

    impl Clock for FixedClock {
        fn now(&self) -> JstDateTime {
            self.0
        }
    }

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
        let completed_at = crate::domain::timezone::JstDateTime::now();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: Some(completed_at),
        };

        let trial = run(trial, command).unwrap();

        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.completed_at(), Some(&completed_at));
    }

    #[test]
    fn test_run_defaults_completed_at_to_clock_now_when_unspecified() {
        let (trial, step_id) = trial_with_step();
        let fixed_now = JstDateTime::now() - chrono::Duration::hours(3);
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };

        let trial = run_with_clock(trial, command, &FixedClock(fixed_now)).unwrap();

        let step = trial.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.completed_at(), Some(&fixed_now));
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
