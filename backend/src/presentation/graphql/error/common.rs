//! GraphQL エラー変換の共通部品

use async_graphql::ErrorExtensions;

/// GraphQL エラーのラッパー
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQLError {
    message: String,
    code: String,
}

impl GraphQLError {
    pub fn new(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
        }
    }
}

impl ErrorExtensions for GraphQLError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(self.message.clone()).extend_with(|_err, e| {
            e.set("code", self.code.clone());
        })
    }
}

/// ユーザー向けエラーメッセージとエラーコードを拡張する
pub trait UserFacingError {
    fn to_user_facing(&self) -> GraphQLError;
}

/// 競合エラー
///
/// 一意制約違反など、同じ対象への並行操作によって発生する競合。
/// 内部エラーではなくリトライで解消し得るため、その旨をユーザーに伝える。
///
/// ユーザーには詳細を見せないが、本番で競合が多発したときに
/// 「どのエンティティのどの制約か」を追えるようログには残す。
///
/// Project / Trial いずれのユースケースからも同じ文言を返すため共通部品とする。
pub fn conflict_error(entity: &str, field: &str) -> GraphQLError {
    log::warn!("Conflict: {}.{}", entity, field);
    GraphQLError::new("他の操作と競合しました。もう一度お試しください", "CONFLICT")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 競合エラーはリトライを促す CONFLICT コードを返す
    #[test]
    fn test_conflict_error_returns_conflict_code() {
        assert_eq!(
            conflict_error("step", "trial_id_position"),
            GraphQLError::new("他の操作と競合しました。もう一度お試しください", "CONFLICT")
        );
    }
}
