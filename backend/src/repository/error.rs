//! sqlx エラー -> RepositoryError 変換
//!
//! sqlx のエラーを一律 `RepositoryError::Internal` に畳むと、ユーザー操作で到達し得る
//! 制約違反（同一 Trial への並行 addStep による position の重複など）まで
//! 内部エラーとして扱われてしまう。
//! SQLSTATE を見て、呼び出し側が意味を判断できるエラーへ振り分ける。

use crate::ports::error::RepositoryError;

/// 一意制約違反の SQLSTATE
const UNIQUE_VIOLATION: &str = "23505";

/// 外部キー制約違反の SQLSTATE
const FOREIGN_KEY_VIOLATION: &str = "23503";

/// 制約名から情報を取り出せなかった場合のプレースホルダー
const UNKNOWN: &str = "unknown";

/// インデックス名によく使われる接頭辞
const INDEX_PREFIXES: [&str; 3] = ["idx_", "uq_", "unique_"];

/// 一意制約・インデックス名によく使われる接尾辞
const UNIQUE_SUFFIXES: [&str; 3] = ["_key", "_unique", "_idx"];

/// sqlx のエラーを `RepositoryError` に変換する
///
/// `entity` には問い合わせ対象のエンティティ名（"trial" / "step" / "parameter" / "project"）を渡す。
/// 制約名の接頭辞除去にも利用するため、テーブル名ではなく単数形のエンティティ名を渡すこと。
pub fn map_sqlx_error(error: sqlx::Error, entity: &str) -> RepositoryError {
    let sqlx::Error::Database(db_error) = &error else {
        return RepositoryError::Internal {
            message: error.to_string(),
        };
    };

    match db_error.code().as_deref() {
        // 一意制約違反はユーザー操作の競合であり内部エラーではない
        Some(UNIQUE_VIOLATION) => RepositoryError::Conflict {
            entity: entity.to_string(),
            field: conflict_field(db_error.constraint(), entity),
        },
        // 外部キー違反は参照先が存在しない（並行して削除された）ことを意味する
        Some(FOREIGN_KEY_VIOLATION) => RepositoryError::NotFound {
            entity: referenced_entity(db_error.constraint(), entity),
            // 欠落している行の値は構造化された形で取得できないため特定しない
            id: UNKNOWN.to_string(),
        },
        _ => RepositoryError::Internal {
            message: error.to_string(),
        },
    }
}

/// 一意制約名から違反したフィールド名を導く
///
/// 例: `steps_trial_id_position_key` (entity: step) -> `trial_id_position`
///     `idx_projects_name` (entity: project) -> `name`
///     `projects_pkey` -> `id`
///
/// 既知のパターンに当てはまらない場合は制約名をそのまま返す。
fn conflict_field(constraint: Option<&str>, entity: &str) -> String {
    let Some(constraint) = constraint else {
        return UNKNOWN.to_string();
    };

    // 主キー制約名はカラム名を含まないため id とみなす
    if constraint.ends_with("_pkey") {
        return "id".to_string();
    }

    let field = strip_prefixes(constraint, &INDEX_PREFIXES);
    let field = strip_table_prefix(field, entity);
    let field = strip_suffixes(field, &UNIQUE_SUFFIXES);

    if field.is_empty() {
        constraint.to_string()
    } else {
        field.to_string()
    }
}

/// 外部キー制約名から参照先のエンティティ名を導く
///
/// 例: `trials_project_id_fkey` (entity: trial) -> `project`
///     `steps_trial_id_fkey` (entity: step) -> `trial`
///
/// 既知のパターンに当てはまらない場合は呼び出し元が渡した entity を返す。
fn referenced_entity(constraint: Option<&str>, entity: &str) -> String {
    let Some(name) = constraint.and_then(|c| c.strip_suffix("_fkey")) else {
        return entity.to_string();
    };

    let name = strip_table_prefix(name, entity);
    let name = name.strip_suffix("_id").unwrap_or(name);

    if name.is_empty() {
        entity.to_string()
    } else {
        name.to_string()
    }
}

/// 先頭に一致する接頭辞をひとつ取り除く
fn strip_prefixes<'a>(name: &'a str, prefixes: &[&str]) -> &'a str {
    prefixes
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .unwrap_or(name)
}

/// 末尾に一致する接尾辞をひとつ取り除く
fn strip_suffixes<'a>(name: &'a str, suffixes: &[&str]) -> &'a str {
    suffixes
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
}

/// テーブル名に相当する接頭辞（`steps_` / `step_`）を取り除く
fn strip_table_prefix<'a>(name: &'a str, entity: &str) -> &'a str {
    [format!("{}s_", entity), format!("{}_", entity)]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix.as_str()))
        .unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::error::{DatabaseError, ErrorKind};
    use std::borrow::Cow;
    use std::error::Error as StdError;
    use std::fmt;

    /// テスト用の `DatabaseError` 実装
    ///
    /// 実際に PostgreSQL へ接続せず、SQLSTATE と制約名の組み合わせを検証するために使用する。
    #[derive(Debug)]
    struct FakeDatabaseError {
        code: String,
        constraint: Option<String>,
    }

    impl fmt::Display for FakeDatabaseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "fake database error ({})", self.code)
        }
    }

    impl StdError for FakeDatabaseError {}

    impl DatabaseError for FakeDatabaseError {
        fn message(&self) -> &str {
            "fake database error"
        }

        fn code(&self) -> Option<Cow<'_, str>> {
            Some(Cow::Borrowed(&self.code))
        }

        fn constraint(&self) -> Option<&str> {
            self.constraint.as_deref()
        }

        fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    fn db_error(code: &str, constraint: Option<&str>) -> sqlx::Error {
        sqlx::Error::Database(Box::new(FakeDatabaseError {
            code: code.to_string(),
            constraint: constraint.map(|c| c.to_string()),
        }))
    }

    #[test]
    fn test_unique_violation_maps_to_conflict_with_field_from_constraint() {
        let error = db_error(UNIQUE_VIOLATION, Some("steps_trial_id_position_key"));

        assert_eq!(
            map_sqlx_error(error, "step"),
            RepositoryError::Conflict {
                entity: "step".to_string(),
                field: "trial_id_position".to_string(),
            }
        );
    }

    #[test]
    fn test_unique_violation_on_unique_index_maps_to_conflict() {
        let error = db_error(UNIQUE_VIOLATION, Some("idx_projects_name"));

        assert_eq!(
            map_sqlx_error(error, "project"),
            RepositoryError::Conflict {
                entity: "project".to_string(),
                field: "name".to_string(),
            }
        );
    }

    #[test]
    fn test_unique_violation_on_primary_key_maps_to_id_field() {
        let error = db_error(UNIQUE_VIOLATION, Some("projects_pkey"));

        assert_eq!(
            map_sqlx_error(error, "project"),
            RepositoryError::Conflict {
                entity: "project".to_string(),
                field: "id".to_string(),
            }
        );
    }

    #[test]
    fn test_unique_violation_with_unknown_constraint_keeps_constraint_name() {
        let error = db_error(UNIQUE_VIOLATION, Some("legacy_constraint"));

        assert_eq!(
            map_sqlx_error(error, "project"),
            RepositoryError::Conflict {
                entity: "project".to_string(),
                field: "legacy_constraint".to_string(),
            }
        );
    }

    #[test]
    fn test_unique_violation_without_constraint_falls_back_to_unknown() {
        let error = db_error(UNIQUE_VIOLATION, None);

        assert_eq!(
            map_sqlx_error(error, "trial"),
            RepositoryError::Conflict {
                entity: "trial".to_string(),
                field: "unknown".to_string(),
            }
        );
    }

    #[test]
    fn test_foreign_key_violation_maps_to_not_found_of_referenced_entity() {
        let error = db_error(FOREIGN_KEY_VIOLATION, Some("trials_project_id_fkey"));

        assert_eq!(
            map_sqlx_error(error, "trial"),
            RepositoryError::NotFound {
                entity: "project".to_string(),
                id: "unknown".to_string(),
            }
        );
    }

    #[test]
    fn test_foreign_key_violation_on_step_maps_to_trial_not_found() {
        let error = db_error(FOREIGN_KEY_VIOLATION, Some("steps_trial_id_fkey"));

        assert_eq!(
            map_sqlx_error(error, "step"),
            RepositoryError::NotFound {
                entity: "trial".to_string(),
                id: "unknown".to_string(),
            }
        );
    }

    #[test]
    fn test_foreign_key_violation_without_constraint_falls_back_to_entity() {
        let error = db_error(FOREIGN_KEY_VIOLATION, None);

        assert_eq!(
            map_sqlx_error(error, "parameter"),
            RepositoryError::NotFound {
                entity: "parameter".to_string(),
                id: "unknown".to_string(),
            }
        );
    }

    #[test]
    fn test_other_sqlstate_maps_to_internal() {
        let error = db_error("42P01", None);
        let message = error.to_string();

        assert_eq!(
            map_sqlx_error(error, "trial"),
            RepositoryError::Internal { message }
        );
    }

    #[test]
    fn test_non_database_error_maps_to_internal() {
        let error = sqlx::Error::RowNotFound;
        let message = error.to_string();

        assert_eq!(
            map_sqlx_error(error, "trial"),
            RepositoryError::Internal { message }
        );
    }
}
