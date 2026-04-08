//! complete_trial アクション
//!
//! Trial を完了ステータスに変更する。

use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::trial_status_validator;

pub use trial_status_validator::Error;

/// バリデーション
pub fn validate(state: &Trial) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    Ok(())
}

/// 状態遷移（validate 成功前提）
pub fn execute(mut state: Trial) -> Trial {
    state.complete();
    state
}

/// validate + execute
pub fn run(state: Trial) -> Result<Trial, Error> {
    validate(&state)?;
    Ok(execute(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::TrialStatus;

    fn in_progress_trial() -> Trial {
        Trial::new(ProjectId::new(), None, None)
    }

    #[test]
    fn test_complete_trial_success() {
        let trial = in_progress_trial();
        let result = run(trial);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_complete_trial_with_steps() {
        let project_id = ProjectId::new();
        let mut trial = Trial::new(project_id, None, None);
        let step = Step::new(trial.id().clone(), "捏ね".to_string(), 0);
        trial.add_step(step);
        let result = run(trial);
        assert!(result.is_ok());
    }

    #[test]
    fn test_returns_error_when_already_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete();
        let result = run(trial);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }
}
