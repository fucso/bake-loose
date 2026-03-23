//! create_trial アクション：新しい Trial を作成する

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;

pub struct Command {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// Trial 作成時のバリデーションエラー
/// 現在は Trial 自体のバリデーションエラーはないが、将来の拡張に備えて定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {}

pub fn validate(_command: &Command) -> Result<(), Error> {
    Ok(())
}

pub fn execute(command: Command) -> Trial {
    Trial::new(command.project_id, command.name, command.memo)
}

pub fn run(command: Command) -> Result<Trial, Error> {
    validate(&command)?;
    Ok(execute(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::trial::TrialStatus;

    fn make_project_id() -> ProjectId {
        ProjectId::new()
    }

    #[test]
    fn test_create_trial() {
        let command = Command {
            project_id: make_project_id(),
            name: Some("バゲット第1回".to_string()),
            memo: Some("初めての試行".to_string()),
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.name(), Some("バゲット第1回"));
        assert_eq!(trial.memo(), Some("初めての試行"));
        assert!(trial.steps().is_empty());
    }

    #[test]
    fn test_trial_status_is_in_progress() {
        let command = Command {
            project_id: make_project_id(),
            name: None,
            memo: None,
        };
        let trial = run(command).unwrap();
        assert_eq!(trial.status(), &TrialStatus::InProgress);
    }
}
