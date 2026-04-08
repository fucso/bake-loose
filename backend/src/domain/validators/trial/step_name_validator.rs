//! Step 名の検証

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyStepName,
}

/// Step 名が空でないことを検証（add_step 用）
pub fn require_not_empty(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::EmptyStepName);
    }
    Ok(())
}

/// オプショナルな Step 名を検証（update_step 用）
/// None の場合はパス、Some の場合は空でないことを検証
pub fn validate_optional(name: &Option<String>) -> Result<(), Error> {
    if let Some(n) = name {
        if n.trim().is_empty() {
            return Err(Error::EmptyStepName);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod require_not_empty {
        use super::*;

        #[test]
        fn test_valid_name_passes() {
            assert!(require_not_empty("捏ね").is_ok());
        }

        #[test]
        fn test_empty_string_returns_error() {
            assert_eq!(require_not_empty(""), Err(Error::EmptyStepName));
        }

        #[test]
        fn test_whitespace_only_returns_error() {
            assert_eq!(require_not_empty("   "), Err(Error::EmptyStepName));
        }

        #[test]
        fn test_whitespace_with_tabs_returns_error() {
            assert_eq!(require_not_empty("\t\n  "), Err(Error::EmptyStepName));
        }

        #[test]
        fn test_name_with_leading_whitespace_passes() {
            assert!(require_not_empty("  捏ね").is_ok());
        }
    }

    mod validate_optional {
        use super::*;

        #[test]
        fn test_none_passes() {
            assert!(validate_optional(&None).is_ok());
        }

        #[test]
        fn test_some_valid_name_passes() {
            assert!(validate_optional(&Some("捏ね".to_string())).is_ok());
        }

        #[test]
        fn test_some_empty_string_returns_error() {
            assert_eq!(
                validate_optional(&Some("".to_string())),
                Err(Error::EmptyStepName)
            );
        }

        #[test]
        fn test_some_whitespace_only_returns_error() {
            assert_eq!(
                validate_optional(&Some("   ".to_string())),
                Err(Error::EmptyStepName)
            );
        }
    }
}
