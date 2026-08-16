//! Trial の名前検証

const MAX_NAME_LENGTH: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,
    NameTooLong { max: usize, actual: usize },
}

/// Trial 名が指定されている場合に、空文字でなく上限文字数以内であることを検証する
///
/// Trial 名は任意項目のため、未指定（`None`）の場合は検証をスキップする。
pub fn validate(name: Option<&str>) -> Result<(), Error> {
    let Some(name) = name else {
        return Ok(());
    };
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
            (None, Ok(())),
            (Some("焼成温度検証".to_string()), Ok(())),
            (Some("a".repeat(MAX_NAME_LENGTH)), Ok(())),
            (Some("".to_string()), Err(Error::EmptyName)),
            (Some("   ".to_string()), Err(Error::EmptyName)),
            (
                Some("a".repeat(MAX_NAME_LENGTH + 1)),
                Err(Error::NameTooLong {
                    max: MAX_NAME_LENGTH,
                    actual: MAX_NAME_LENGTH + 1,
                }),
            ),
        ];

        for (name, expected) in cases {
            assert_eq!(validate(name.as_deref()), expected);
        }
    }
}
