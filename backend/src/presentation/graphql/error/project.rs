//! Project 関連のユースケースエラー -> GraphQL エラー変換

use async_graphql::ErrorExtensions;

use crate::domain::actions::project::create_project as create_project_action;
use crate::presentation::graphql::error::common::{conflict_error, GraphQLError, UserFacingError};
use crate::use_case::project::{create_project, get_project, list_projects};

impl UserFacingError for get_project::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            get_project::Error::Infrastructure(e) => {
                // インフラエラーの詳細は隠蔽
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<get_project::Error> for async_graphql::Error {
    fn from(e: get_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for create_project::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            create_project::Error::Domain(e) => match e {
                create_project_action::Error::EmptyName => {
                    GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR")
                }
                create_project_action::Error::NameTooLong { max, .. } => GraphQLError::new(
                    format!("{}文字以内で入力してください", max),
                    "VALIDATION_ERROR",
                ),
            },
            create_project::Error::DuplicateName => {
                GraphQLError::new("同じ名前のプロジェクトが既に存在します", "DUPLICATE_ERROR")
            }
            create_project::Error::Conflict { entity, field } => conflict_error(entity, field),
            create_project::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<create_project::Error> for async_graphql::Error {
    fn from(e: create_project::Error) -> Self {
        e.to_user_facing().extend()
    }
}

impl UserFacingError for list_projects::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            list_projects::Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}

impl From<list_projects::Error> for async_graphql::Error {
    fn from(e: list_projects::Error) -> Self {
        e.to_user_facing().extend()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// name 以外の一意制約違反は CONFLICT として返す
    ///
    /// 以前は Infrastructure に畳まれて INTERNAL_ERROR になっていたが、
    /// DB が検出した制約違反はサーバー内部の障害ではないため、
    /// Trial 側と同じくリトライを促す CONFLICT に揃える。
    #[test]
    fn test_conflict_maps_to_conflict_code() {
        let error = create_project::Error::Conflict {
            entity: "project".to_string(),
            field: "slug".to_string(),
        }
        .to_user_facing();

        assert_eq!(
            error,
            GraphQLError::new("他の操作と競合しました。もう一度お試しください", "CONFLICT")
        );
        assert_ne!(
            error,
            GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
        );
    }

    /// 名前の重複は従来どおり DUPLICATE_ERROR のまま
    #[test]
    fn test_duplicate_name_still_maps_to_duplicate_error() {
        assert_eq!(
            create_project::Error::DuplicateName.to_user_facing(),
            GraphQLError::new("同じ名前のプロジェクトが既に存在します", "DUPLICATE_ERROR")
        );
    }
}
