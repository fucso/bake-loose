use uuid::Uuid;

use crate::domain::models::project::{Project, ProjectId};

const MAX_NAME_LENGTH: usize = 100;

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
    if command.name.chars().count() > MAX_NAME_LENGTH {
        return Err(Error::NameTooLong {
            max: MAX_NAME_LENGTH,
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
    fn test_name_validation() {
        let cases = vec![
            ("a".repeat(MAX_NAME_LENGTH), Ok(())),
            ("".to_string(), Err(Error::EmptyName)),
            ("   ".to_string(), Err(Error::EmptyName)),
            (
                "a".repeat(MAX_NAME_LENGTH + 1),
                Err(Error::NameTooLong {
                    max: MAX_NAME_LENGTH,
                    actual: MAX_NAME_LENGTH + 1,
                }),
            ),
        ];

        for (name, expected) in cases {
            let command = Command {
                name: name.to_string(),
            };
            let result = validate(&command);
            assert_eq!(result, expected);
        }
    }
}
