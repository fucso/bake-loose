use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::Trial;

pub struct Command {
    pub project_id: ProjectId,
    pub name: Option<String>,
    pub memo: Option<String>,
}

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

    #[test]
    fn test_run_creates_trial_in_progress_with_no_steps() {
        let command = Command {
            project_id: ProjectId::new(),
            name: Some("焼成温度検証".to_string()),
            memo: Some("初回".to_string()),
        };
        let trial = run(command).unwrap();
        assert_eq!(
            trial.status(),
            &crate::domain::models::trial::TrialStatus::InProgress
        );
        assert!(trial.steps().is_empty());
        assert_eq!(trial.name(), Some("焼成温度検証"));
        assert_eq!(trial.memo(), Some("初回"));
    }

    #[test]
    fn test_execute_generates_unique_id_and_links_project() {
        let project_id = ProjectId::new();
        let command1 = Command {
            project_id: project_id.clone(),
            name: None,
            memo: None,
        };
        let command2 = Command {
            project_id: project_id.clone(),
            name: None,
            memo: None,
        };
        let trial1 = execute(command1);
        let trial2 = execute(command2);
        assert_ne!(trial1.id(), trial2.id());
        assert_eq!(trial1.project_id(), &project_id);
    }

    #[test]
    fn test_execute_allows_no_name_and_memo() {
        let command = Command {
            project_id: ProjectId::new(),
            name: None,
            memo: None,
        };
        let trial = execute(command);
        assert_eq!(trial.name(), None);
        assert_eq!(trial.memo(), None);
    }
}
