//! Step の名前検証

use crate::domain::errors::trial_error::Error;

const MAX_NAME_LENGTH: usize = 100;

/// Step 名が空文字でなく上限文字数以内であることを検証する
pub fn validate(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::EmptyStepName);
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(Error::StepNameTooLong {
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
            ("こね".to_string(), Ok(())),
            ("a".repeat(MAX_NAME_LENGTH), Ok(())),
            ("".to_string(), Err(Error::EmptyStepName)),
            ("   ".to_string(), Err(Error::EmptyStepName)),
            (
                "a".repeat(MAX_NAME_LENGTH + 1),
                Err(Error::StepNameTooLong {
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
