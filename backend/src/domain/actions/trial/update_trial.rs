use crate::domain::models::trial::{Trial, TrialStatus};

pub struct Command {
    pub name: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    AlreadyCompleted,
}

/// バリデーション
pub fn validate(state: &Trial, _command: &Command) -> Result<(), Error> {
    if state.status() == &TrialStatus::Completed {
        return Err(Error::AlreadyCompleted);
    }
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
        let mut trial = Trial::new(
            ProjectId::new(),
            Some("完了済み試行".to_string()),
            None,
        );
        trial.complete();
        trial
    }

    #[test]
    fn test_update_name() {
        let trial = make_in_progress_trial();
        let command = Command {
            name: Some("新しい名前".to_string()),
            memo: None,
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.name(), Some("新しい名前"));
    }

    #[test]
    fn test_update_memo() {
        let trial = make_in_progress_trial();
        let command = Command {
            name: None,
            memo: Some("新しいメモ".to_string()),
        };
        let result = run(trial, command).unwrap();
        assert_eq!(result.memo(), Some("新しいメモ"));
    }

    #[test]
    fn test_update_both_name_and_memo() {
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
    fn test_updated_at_is_changed() {
        use std::thread::sleep;
        use std::time::Duration;

        let trial = make_in_progress_trial();
        let original_updated_at = *trial.updated_at();
        sleep(Duration::from_millis(10));
        let command = Command {
            name: Some("更新された名前".to_string()),
            memo: None,
        };
        let result = run(trial, command).unwrap();
        assert!(result.updated_at() > &original_updated_at);
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let trial = make_completed_trial();
        let command = Command {
            name: Some("更新試行".to_string()),
            memo: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::AlreadyCompleted));
    }
}
