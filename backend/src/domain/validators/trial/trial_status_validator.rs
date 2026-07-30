//! Trial のステータス検証

use crate::domain::models::trial::{Trial, TrialStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
}

/// Trial が InProgress であることを検証する
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

    #[test]
    fn test_require_in_progress_ok_when_in_progress() {
        let trial = Trial::new(ProjectId::new(), None, None);
        assert_eq!(require_in_progress(&trial), Ok(()));
    }

    #[test]
    fn test_require_in_progress_err_when_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.complete();
        assert_eq!(
            require_in_progress(&trial),
            Err(Error::TrialAlreadyCompleted)
        );
    }
}
