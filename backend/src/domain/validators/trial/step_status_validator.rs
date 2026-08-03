//! Step のステータス検証

use crate::domain::models::step::Step;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    StepAlreadyCompleted,
}

/// Step が未完了（InProgress）であることを検証する
pub fn require_in_progress(step: &Step) -> Result<(), Error> {
    if step.is_completed() {
        return Err(Error::StepAlreadyCompleted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::step::StepId;
    use crate::domain::models::trial::TrialId;

    #[test]
    fn test_require_in_progress_ok_when_not_completed() {
        let step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        assert_eq!(require_in_progress(&step), Ok(()));
    }

    #[test]
    fn test_require_in_progress_err_when_completed() {
        let step = Step::from_raw(
            StepId::new(),
            TrialId::new(),
            "こね".to_string(),
            0,
            Some(crate::domain::timezone::JstDateTime::now()),
            Some(crate::domain::timezone::JstDateTime::now()),
            Vec::new(),
        );
        assert_eq!(require_in_progress(&step), Err(Error::StepAlreadyCompleted));
    }
}
