use crate::domain::models::project::Project;
use crate::domain::validators::project::project_name_validator;

pub use crate::domain::errors::project_error::Error;

pub struct Command {
    pub name: String,
}

pub fn validate(command: &Command) -> Result<(), Error> {
    project_name_validator::validate(&command.name)?;
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
    fn test_run_err_when_name_is_empty() {
        let command = Command {
            name: "".to_string(),
        };

        assert_eq!(run(command), Err(Error::EmptyName));
    }

    #[test]
    fn test_run_err_when_name_too_long() {
        let command = Command {
            name: "a".repeat(101),
        };

        assert_eq!(
            run(command),
            Err(Error::NameTooLong {
                max: 100,
                actual: 101,
            })
        );
    }
}
