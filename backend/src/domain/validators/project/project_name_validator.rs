//! Project の名前検証

use crate::domain::errors::project_error::Error;

const MAX_NAME_LENGTH: usize = 100;

/// Project 名が空文字でなく上限文字数以内であることを検証する
pub fn validate(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::EmptyName);
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(Error::NameTooLong {
            max: MAX_NAME_LENGTH,
            actual: name.chars().count(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        let cases = vec![
            ("パン作り".to_string(), Ok(())),
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
            assert_eq!(validate(&name), expected);
        }
    }
}
