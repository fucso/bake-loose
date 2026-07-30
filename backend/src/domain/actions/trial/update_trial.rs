//! Trial の name/memo を更新するアクション

use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::trial_status_validator;

pub use trial_status_validator::Error;

/// 指定したフィールドのみを部分更新する（`None` は未指定＝変更なし）
pub struct Command {
    pub name: Option<Option<String>>,
    pub memo: Option<Option<String>>,
}

/// バリデーション
pub fn validate(state: &Trial) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    if let Some(name) = command.name {
        state.set_name(name);
    }
    if let Some(memo) = command.memo {
        state.set_memo(memo);
    }
    state
}

/// validate + execute
pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;

    fn in_progress_trial() -> Trial {
        Trial::new(
            ProjectId::new(),
            Some("元の名前".to_string()),
            Some("元のメモ".to_string()),
        )
    }

    #[test]
    fn test_run_updates_name_and_memo_when_in_progress() {
        let trial = in_progress_trial();
        let command = Command {
            name: Some(Some("新しい名前".to_string())),
            memo: Some(Some("新しいメモ".to_string())),
        };

        let updated = run(trial, command).unwrap();

        assert_eq!(updated.name(), Some("新しい名前"));
        assert_eq!(updated.memo(), Some("新しいメモ"));
    }

    #[test]
    fn test_run_partially_updates_only_specified_fields() {
        let trial = in_progress_trial();
        let command = Command {
            name: Some(Some("新しい名前".to_string())),
            memo: None,
        };

        let updated = run(trial, command).unwrap();

        assert_eq!(updated.name(), Some("新しい名前"));
        assert_eq!(updated.memo(), Some("元のメモ"));
    }

    #[test]
    fn test_run_can_clear_name_and_memo_to_none() {
        let trial = in_progress_trial();
        let command = Command {
            name: Some(None),
            memo: Some(None),
        };

        let updated = run(trial, command).unwrap();

        assert_eq!(updated.name(), None);
        assert_eq!(updated.memo(), None);
    }

    #[test]
    fn test_run_rejects_update_when_trial_completed() {
        let mut trial = in_progress_trial();
        trial.complete();
        let command = Command {
            name: Some(Some("新しい名前".to_string())),
            memo: None,
        };

        let result = run(trial, command);

        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }
}
