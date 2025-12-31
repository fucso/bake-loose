use uuid::Uuid;

use crate::domain::models::project::{Project, ProjectId};

pub struct Command {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,
    NameTooLong { max: usize, actual: usize },
}

pub fn validate(command: &Command) -> Result<(), Error> {
    if command.name.trim().is_empty() {
        return Err(Error::EmptyName);
    }
    if command.name.chars().count() > 100 {
        return Err(Error::NameTooLong {
            max: 100,
            actual: command.name.chars().count(),
        });
    }
    Ok(())
}

pub fn execute(command: Command) -> Project {
    Project::new(command.name)
}

pub fn run(command: Command) -> Result<Project, Error> {
    validate(&command)?;
    Ok(execute(command))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_creates_project_with_valid_name() {
        let command = Command {
            name: "Test Project".to_string(),
        };
        let project = run(command).unwrap();
        assert_eq!(project.name(), "Test Project");
    }

    #[test]
    fn test_run_creates_project_with_max_length_name() {
        let name = "a".repeat(100);
        let command = Command { name: name.clone() };
        let project = run(command).unwrap();
        assert_eq!(project.name(), &name);
    }

    #[test]
    fn test_execute_generates_unique_id() {
        let command1 = Command {
            name: "Project 1".to_string(),
        };
        let command2 = Command {
            name: "Project 2".to_string(),
        };
        let project1 = execute(command1);
        let project2 = execute(command2);
        assert_ne!(project1.id(), project2.id());
    }

    #[test]
    fn test_validate_returns_error_for_empty_name() {
        let command = Command {
            name: "".to_string(),
        };
        let result = validate(&command);
        assert_eq!(result, Err(Error::EmptyName));
    }

    #[test]
    fn test_validate_returns_error_for_whitespace_only_name() {
        let command = Command {
            name: "   ".to_string(),
        };
        let result = validate(&command);
        assert_eq!(result, Err(Error::EmptyName));
    }

    #[test]
    fn test_validate_returns_error_for_too_long_name() {
        let name = "a".repeat(101);
        let command = Command { name: name.clone() };
        let result = validate(&command);
        assert_eq!(
            result,
            Err(Error::NameTooLong {
                max: 100,
                actual: 101
            })
        );
    }
}
