//! complete_trial アクション
//!
//! Trial を完了ステータスに変更する。

use chrono::Utc;

use crate::domain::models::trial::{Trial, TrialStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    AlreadyCompleted,
}

/// バリデーション
pub fn validate(state: &Trial) -> Result<(), Error> {
    if state.status() == &TrialStatus::Completed {
        return Err(Error::AlreadyCompleted);
    }
    Ok(())
}

/// 状態遷移（validate 成功前提）
pub fn execute(state: Trial) -> Trial {
    Trial::from_raw(
        state.id().clone(),
        state.project_id().clone(),
        state.name().map(|s| s.to_string()),
        state.memo().map(|s| s.to_string()),
        TrialStatus::Completed,
        state.steps().to_vec(),
        *state.created_at(),
        Utc::now(),
    )
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
    fn test_complete_trial() {
        let trial = in_progress_trial();
        let result = run(trial);
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_is_completed() {
        let trial = in_progress_trial();
        let result = run(trial).unwrap();
        assert_eq!(result.status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_updated_at_is_changed() {
        let trial = in_progress_trial();
        let original_updated_at = *trial.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result = run(trial).unwrap();
        assert!(result.updated_at() > &original_updated_at);
    }

    #[test]
    fn test_complete_trial_with_incomplete_steps() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id.clone(), None, None);
        let step = Step::new(trial.id().clone(), "捏ね".to_string(), 0);
        let trial_with_steps = Trial::from_raw(
            trial.id().clone(),
            project_id,
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            TrialStatus::InProgress,
            vec![step],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let result = run(trial_with_steps);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_returns_error_when_already_completed() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id.clone(), None, None);
        let completed_trial = Trial::from_raw(
            trial.id().clone(),
            project_id,
            trial.name().map(|s| s.to_string()),
            trial.memo().map(|s| s.to_string()),
            TrialStatus::Completed,
            vec![],
            *trial.created_at(),
            *trial.updated_at(),
        );
        let result = run(completed_trial);
        assert_eq!(result, Err(Error::AlreadyCompleted));
    }
}
