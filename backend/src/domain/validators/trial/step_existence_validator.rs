//! Step の存在検証

use crate::domain::models::step::{Step, StepId};
use crate::domain::models::trial::Trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    StepNotFound,
}

/// 指定された Step が Trial 内に存在することを検証
pub fn require_exists<'a>(trial: &'a Trial, step_id: &StepId) -> Result<&'a Step, Error> {
    trial
        .steps()
        .iter()
        .find(|s| s.id() == step_id)
        .ok_or(Error::StepNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;
    use crate::domain::models::trial::Trial;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "テストステップ".to_string(), 0);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[test]
    fn test_existing_step_returns_step() {
        let (trial, step_id) = trial_with_step();
        let result = require_exists(&trial, &step_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), &step_id);
    }

    #[test]
    fn test_non_existing_step_returns_error() {
        let (trial, _) = trial_with_step();
        let non_existent_id = StepId::new();
        assert_eq!(
            require_exists(&trial, &non_existent_id),
            Err(Error::StepNotFound)
        );
    }

    #[test]
    fn test_empty_trial_returns_error() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let step_id = StepId::new();
        assert_eq!(require_exists(&trial, &step_id), Err(Error::StepNotFound));
    }
}
