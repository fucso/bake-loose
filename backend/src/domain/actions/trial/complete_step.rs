//! complete_step アクション - Step を完了状態にする

use chrono::{DateTime, Utc};

use crate::domain::models::step::{Step, StepId};
use crate::domain::models::trial::{Trial, TrialStatus};

/// complete_step コマンド
pub struct Command {
    pub step_id: StepId,
    pub completed_at: Option<DateTime<Utc>>,
}

/// complete_step エラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    if state.status() == &TrialStatus::Completed {
        return Err(Error::TrialAlreadyCompleted);
    }
    let step = state.steps().iter().find(|s| s.id() == &command.step_id);
    match step {
        None => Err(Error::StepNotFound),
        Some(s) if s.completed_at().is_some() => Err(Error::StepAlreadyCompleted),
        _ => Ok(()),
    }
}

/// 状態遷移（validate 成功前提）
pub fn execute(state: Trial, command: Command) -> Trial {
    let now = Utc::now();
    let completed_at = command.completed_at.unwrap_or(now);

    let steps = state
        .steps()
        .iter()
        .map(|s| {
            if s.id() == &command.step_id {
                Step::from_raw(
                    s.id().clone(),
                    s.trial_id().clone(),
                    s.name().to_string(),
                    s.position(),
                    s.started_at().cloned(),
                    Some(completed_at),
                    s.parameters().to_vec(),
                    *s.created_at(),
                    now,
                )
            } else {
                s.clone()
            }
        })
        .collect();

    Trial::from_raw(
        state.id().clone(),
        state.project_id().clone(),
        state.name().map(|s| s.to_string()),
        state.memo().map(|s| s.to_string()),
        state.status().clone(),
        steps,
        *state.created_at(),
        now,
    )
}

/// validate + execute
pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::StepId;
    use crate::domain::models::trial::{Trial, TrialId, TrialStatus};

    fn make_trial_with_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let past = Utc::now() - chrono::Duration::seconds(10);

        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "捏ね".to_string(),
            0,
            None,
            None,
            vec![],
            past,
            past,
        );

        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::InProgress,
            vec![step],
            past,
            past,
        );

        (trial, step_id)
    }

    fn make_completed_trial_with_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let past = Utc::now() - chrono::Duration::seconds(10);

        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "捏ね".to_string(),
            0,
            None,
            None,
            vec![],
            past,
            past,
        );

        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::Completed,
            vec![step],
            past,
            past,
        );

        (trial, step_id)
    }

    fn make_trial_with_already_completed_step() -> (Trial, StepId) {
        let project_id = ProjectId::new();
        let trial_id = TrialId::new();
        let step_id = StepId::new();
        let past = Utc::now() - chrono::Duration::seconds(10);
        let completed_past = Utc::now() - chrono::Duration::seconds(5);

        let step = Step::from_raw(
            step_id.clone(),
            trial_id.clone(),
            "捏ね".to_string(),
            0,
            None,
            Some(completed_past),
            vec![],
            past,
            past,
        );

        let trial = Trial::from_raw(
            trial_id,
            project_id,
            None,
            None,
            TrialStatus::InProgress,
            vec![step],
            past,
            past,
        );

        (trial, step_id)
    }

    #[test]
    fn test_complete_step() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.completed_at().is_some());
    }

    #[test]
    fn test_completed_at_is_set() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.completed_at().is_some());
    }

    #[test]
    fn test_complete_step_with_specified_time() {
        let (trial, step_id) = make_trial_with_step();
        let specified_time = Utc::now() - chrono::Duration::hours(1);
        let command = Command {
            step_id: step_id.clone(),
            completed_at: Some(specified_time),
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.completed_at(), Some(&specified_time));
    }

    #[test]
    fn test_step_updated_at_is_changed() {
        let (trial, step_id) = make_trial_with_step();
        let original_step = trial
            .steps()
            .iter()
            .find(|s| s.id() == &step_id)
            .unwrap()
            .clone();
        let original_updated_at = *original_step.updated_at();

        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_ne!(step.updated_at(), &original_updated_at);
    }

    #[test]
    fn test_trial_updated_at_is_changed() {
        let (trial, step_id) = make_trial_with_step();
        let original_updated_at = *trial.updated_at();

        let command = Command {
            step_id,
            completed_at: None,
        };
        let result = run(trial, command).unwrap();
        assert_ne!(result.updated_at(), &original_updated_at);
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (trial, step_id) = make_completed_trial_with_step();
        let command = Command {
            step_id,
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _) = make_trial_with_step();
        let command = Command {
            step_id: StepId::new(), // non-existent step
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_already_completed() {
        let (trial, step_id) = make_trial_with_already_completed_step();
        let command = Command {
            step_id,
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }
}
