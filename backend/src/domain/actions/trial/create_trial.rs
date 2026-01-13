//! Trial 作成アクション

use crate::domain::models::{ProjectId, Trial};

/// Trial 作成コマンド
pub struct Command {
    pub project_id: ProjectId,
    pub memo: Option<String>,
}

/// エラー型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Project との紐付けがない（project_id が空の UUID）
    EmptyProjectId,
}

/// 出力型
#[derive(Debug)]
pub struct Output {
    pub trial: Trial,
}

/// バリデーション
pub fn validate(command: &Command) -> Result<(), Error> {
    // project_id が空（nil UUID）でないことを確認
    if command.project_id.0.is_nil() {
        return Err(Error::EmptyProjectId);
    }
    Ok(())
}

/// 実行
pub fn execute(command: Command) -> Output {
    let trial = Trial::new(command.project_id, command.memo);
    Output { trial }
}

/// バリデーション + 実行
pub fn run(command: Command) -> Result<Output, Error> {
    validate(&command)?;
    Ok(execute(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::TrialStatus;
    use uuid::Uuid;

    #[test]
    fn test_create_trial_with_memo() {
        let command = Command {
            project_id: ProjectId::new(),
            memo: Some("テストメモ".to_string()),
        };
        let output = run(command).unwrap();
        assert_eq!(output.trial.memo(), Some("テストメモ"));
    }

    #[test]
    fn test_create_trial_without_memo() {
        let command = Command {
            project_id: ProjectId::new(),
            memo: None,
        };
        let output = run(command).unwrap();
        assert!(output.trial.memo().is_none());
    }

    #[test]
    fn test_trial_status_is_in_progress() {
        let command = Command {
            project_id: ProjectId::new(),
            memo: None,
        };
        let output = run(command).unwrap();
        assert_eq!(output.trial.status(), TrialStatus::InProgress);
    }

    #[test]
    fn test_trial_has_correct_project_id() {
        let project_id = ProjectId::new();
        let command = Command {
            project_id: project_id.clone(),
            memo: None,
        };
        let output = run(command).unwrap();
        assert_eq!(output.trial.project_id(), &project_id);
    }

    #[test]
    fn test_returns_error_when_project_id_is_nil() {
        let command = Command {
            project_id: ProjectId(Uuid::nil()),
            memo: None,
        };
        let result = run(command);
        assert_eq!(result.unwrap_err(), Error::EmptyProjectId);
    }
}
