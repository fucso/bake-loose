//! update_trial アクション
//!
//! Trial の名前・メモを更新する。

use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::trial_status_validator;

pub use trial_status_validator::Error;

pub struct Command {
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// バリデーション
pub fn validate(state: &Trial, _command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    if let Some(name) = command.name {
        state.set_name(Some(name));
    }
    if let Some(memo) = command.memo {
        state.set_memo(Some(memo));
    }
    state
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

    fn make_in_progress_trial() -> Trial {
        Trial::new(
            ProjectId::new(),
            Some("テスト試行".to_string()),
            Some("メモ".to_string()),
        )
    }

    fn make_completed_trial() -> Trial {
        let mut trial = Trial::new(ProjectId::new(), Some("完了済み試行".to_string()), None);
        trial.complete();
        trial
    }

    #[test]
    fn test_update_trial_success() {
        let trial = make_in_progress_trial();
        let command = Command {
            name: Some("新しい名前".to_string()),
            memo: Some("新しいメモ".to_string()),
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.name(), Some("新しい名前"));
        assert_eq!(result.memo(), Some("新しいメモ"));
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let trial = make_completed_trial();
        let command = Command {
            name: Some("更新試行".to_string()),
            memo: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }
}
