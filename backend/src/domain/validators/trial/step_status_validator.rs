//! Step のステータス検証

use crate::domain::models::step::Step;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    StepAlreadyCompleted,
}

/// Step が進行中（未完了）であることを検証
pub fn require_in_progress(step: &Step) -> Result<(), Error> {
    if step.completed_at().is_some() {
        return Err(Error::StepAlreadyCompleted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::TrialId;
    use chrono::Utc;

    fn in_progress_step() -> Step {
        Step::new(TrialId::new(), "テストステップ".to_string(), 0)
    }

    fn completed_step() -> Step {
        let mut step = Step::new(TrialId::new(), "完了ステップ".to_string(), 0);
        step.complete(Utc::now());
        step
    }

    #[test]
    fn test_in_progress_step_passes() {
        let step = in_progress_step();
        assert!(require_in_progress(&step).is_ok());
    }

    #[test]
    fn test_completed_step_returns_error() {
        let step = completed_step();
        assert_eq!(
            require_in_progress(&step),
            Err(Error::StepAlreadyCompleted)
        );
    }
}
