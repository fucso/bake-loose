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

    struct UpdateNameMemoCase {
        input_name: Option<Option<&'static str>>,
        input_memo: Option<Option<&'static str>>,
        after_name: Option<&'static str>,
        after_memo: Option<&'static str>,
    }

    #[test]
    fn test_run_updates_name_and_memo() {
        let cases = [
            // 両方指定: 両方更新される
            UpdateNameMemoCase {
                input_name: Some(Some("新しい名前")),
                input_memo: Some(Some("新しいメモ")),
                after_name: Some("新しい名前"),
                after_memo: Some("新しいメモ"),
            },
            // name のみ指定: memo は元の値を維持
            UpdateNameMemoCase {
                input_name: Some(Some("新しい名前")),
                input_memo: None,
                after_name: Some("新しい名前"),
                after_memo: Some("元のメモ"),
            },
            // 両方 Some(None) 指定: 両方 None にクリアされる
            UpdateNameMemoCase {
                input_name: Some(None),
                input_memo: Some(None),
                after_name: None,
                after_memo: None,
            },
        ];

        for case in cases {
            let trial = in_progress_trial();
            let command = Command {
                name: case.input_name.map(|v| v.map(|s| s.to_string())),
                memo: case.input_memo.map(|v| v.map(|s| s.to_string())),
            };

            let updated = run(trial, command).unwrap();

            assert_eq!(updated.name(), case.after_name);
            assert_eq!(updated.memo(), case.after_memo);
        }
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
