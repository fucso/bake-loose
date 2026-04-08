//! Trial のステータス検証

use crate::domain::models::trial::{Trial, TrialStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
}

/// Trial が進行中（InProgress）であることを検証
pub fn require_in_progress(trial: &Trial) -> Result<(), Error> {
    if trial.status() == &TrialStatus::Completed {
        return Err(Error::TrialAlreadyCompleted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialId;
    use chrono::Utc;

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
    fn test_in_progress_trial_passes() {
        let trial = in_progress_trial();
        assert!(require_in_progress(&trial).is_ok());
    }

    #[test]
    fn test_completed_trial_returns_error() {
        let trial = completed_trial();
        assert_eq!(
            require_in_progress(&trial),
            Err(Error::TrialAlreadyCompleted)
        );
    }
}
