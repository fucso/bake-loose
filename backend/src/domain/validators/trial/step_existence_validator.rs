//! Step の存在確認

use crate::domain::models::step::{Step, StepId};
use crate::domain::models::trial::Trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    StepNotFound,
}

/// 指定 ID の Step が Trial に存在することを検証する
pub fn require_exists<'a>(trial: &'a Trial, step_id: &StepId) -> Result<&'a Step, Error> {
    trial
        .steps()
        .iter()
        .find(|step| step.id() == step_id)
        .ok_or(Error::StepNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;

    #[test]
    fn test_require_exists_ok_when_step_exists() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);

        let result = require_exists(&trial, &step_id);
        assert_eq!(result.map(|s| s.id().clone()), Ok(step_id));
    }

    #[test]
    fn test_require_exists_err_when_step_not_found() {
        let trial = Trial::new(ProjectId::new(), None, None);
        let missing_id = StepId::new();

        assert_eq!(
            require_exists(&trial, &missing_id),
            Err(Error::StepNotFound)
        );
    }
}
