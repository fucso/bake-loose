//! Step の存在確認

use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    StepNotFound,
}

/// 指定 ID の Step が Trial に存在することを検証する
///
/// 判定結果のみを返す。実態が必要な場合は呼び出し側（Action の validate）で
/// `state.steps()` から取得する。
pub fn require_exists(trial: &Trial, step_id: &StepId) -> Result<(), Error> {
    if trial.steps().iter().any(|step| step.id() == step_id) {
        Ok(())
    } else {
        Err(Error::StepNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    #[test]
    fn test_require_exists_ok_when_step_exists() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);

        assert_eq!(require_exists(&trial, &step_id), Ok(()));
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
