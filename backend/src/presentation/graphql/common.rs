//! GraphQL リゾルバー共通のヘルパー

use async_graphql::{ErrorExtensions, Result, ID};
use uuid::Uuid;

use crate::presentation::graphql::error::GraphQLError;

/// GraphQL の ID を UUID にパースする
///
/// パースに失敗した場合は種別によらず統一されたメッセージ・コードを返す。
pub fn parse_uuid(id: &ID) -> Result<Uuid> {
    Uuid::parse_str(&id.0)
        .map_err(|_| GraphQLError::new("Invalid UUID format", "VALIDATION_ERROR").extend())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_valid() {
        let id = ID("11111111-1111-1111-1111-111111111111".to_string());
        assert!(parse_uuid(&id).is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid_returns_unified_error() {
        let id = ID("not-a-uuid".to_string());
        let err = parse_uuid(&id).unwrap_err();

        assert_eq!(err.message, "Invalid UUID format");
        assert_eq!(
            err.extensions.as_ref().unwrap().get("code"),
            Some(&async_graphql::Value::from("VALIDATION_ERROR"))
        );
    }
}
