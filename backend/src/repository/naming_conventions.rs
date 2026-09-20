//! DB スキーマ命名規約
//!
//! 本モジュールは「一般的によく使われる名前」ではなく、
//! **このプロジェクトが定めたスキーマ命名規約** を Rust 側の定数・ヘルパーとして表現する。
//! `backend/migrations/` の SQL はこの規約に従って記述すること。
//!
//! 規約の本文は `.claude/rules/backend/repository.md` の「スキーマ命名規約」を参照。
//!
//! # 規約の要点
//!
//! - テーブル名は複数形の snake_case（`projects` / `trials` / `steps` / `parameters`）。
//!   Rust 側のエンティティ名はその単数形（`project` / `trial` / `step` / `parameter`）。
//! - 主キー: `{table}_pkey`（PostgreSQL のデフォルト名をそのまま使う）
//! - 一意制約: `{table}_{columns}_key`（PostgreSQL のデフォルト名をそのまま使う）
//! - 外部キー: `{table}_{column}_fkey`（PostgreSQL のデフォルト名をそのまま使う。
//!   `{table}` は常に **参照する側** のテーブル）
//! - インデックス: `idx_{table}_{columns}` を明示的に付与する。
//!   一意性は `CREATE UNIQUE INDEX` で表現し、`uq_` / `unique_` のような接頭辞では表現しない。
//!
//! `map_sqlx_error` はこの規約を前提に制約名からエンティティ名・フィールド名を復元する。
//! 規約から外れた名前を付けてもコンパイルは通るため、エラー分類だけが静かに劣化する点に注意。

/// インデックス名の接頭辞
///
/// 一意性は `CREATE UNIQUE INDEX` で表現するため、
/// 一意インデックス専用の接頭辞（`uq_` / `unique_` など）は用いない。
pub(crate) const INDEX_PREFIX: &str = "idx_";

/// 主キー制約名の接尾辞（PostgreSQL のデフォルト）
pub(crate) const PRIMARY_KEY_SUFFIX: &str = "_pkey";

/// 一意制約名の接尾辞（PostgreSQL のデフォルト）
pub(crate) const UNIQUE_CONSTRAINT_SUFFIX: &str = "_key";

/// 外部キー制約名の接尾辞（PostgreSQL のデフォルト）
pub(crate) const FOREIGN_KEY_SUFFIX: &str = "_fkey";

/// 制約名・インデックス名から取り除く接頭辞の一覧
///
/// 規約上インデックスの接頭辞は `idx_` のみ。
pub(crate) const INDEX_PREFIXES: [&str; 1] = [INDEX_PREFIX];

/// 一意性に関する制約名から取り除く接尾辞の一覧
///
/// 規約上、一意制約は PostgreSQL デフォルトの `_key` のみ。
pub(crate) const UNIQUE_SUFFIXES: [&str; 1] = [UNIQUE_CONSTRAINT_SUFFIX];

/// 単数形のエンティティ名から、制約名に現れるテーブル接頭辞を組み立てる
///
/// 規約上テーブル名は常に複数形のため、`{entity}s_` のみを生成する。
pub(crate) fn table_prefix(entity: &str) -> String {
    format!("{entity}s_")
}

/// テーブル名に相当する接頭辞（`steps_`）を取り除く
///
/// 規約上テーブル名は常に複数形であるため、**複数形の接頭辞のみ** を対象とする。
/// 単数形（`step_`）も許容すると、`step_notes` のような別テーブルの制約名
/// `step_notes_step_id_fkey` がエンティティ `step` の接頭辞として誤って一致し、
/// 誤ったエンティティ名（`notes_step`）を導いてしまう。
///
/// 一致しない場合は `None` を返す。
pub(crate) fn strip_table_prefix_opt<'a>(name: &'a str, entity: &str) -> Option<&'a str> {
    name.strip_prefix(table_prefix(entity).as_str())
}

/// テーブル名に相当する接頭辞（`steps_`）を取り除く
///
/// 一致しない場合は元の文字列をそのまま返す。
pub(crate) fn strip_table_prefix<'a>(name: &'a str, entity: &str) -> &'a str {
    strip_table_prefix_opt(name, entity).unwrap_or(name)
}

/// 先頭に一致する接頭辞をひとつ取り除く
pub(crate) fn strip_prefixes<'a>(name: &'a str, prefixes: &[&str]) -> &'a str {
    prefixes
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .unwrap_or(name)
}

/// 末尾に一致する接尾辞をひとつ取り除く
pub(crate) fn strip_suffixes<'a>(name: &'a str, suffixes: &[&str]) -> &'a str {
    suffixes
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_prefix_is_plural() {
        assert_eq!(table_prefix("step"), "steps_");
        assert_eq!(table_prefix("project"), "projects_");
        assert_eq!(table_prefix("parameter"), "parameters_");
    }

    #[test]
    fn test_strip_table_prefix_removes_plural_prefix() {
        assert_eq!(
            strip_table_prefix_opt("steps_trial_id_position", "step"),
            Some("trial_id_position")
        );
        assert_eq!(
            strip_table_prefix("trials_project_id", "trial"),
            "project_id"
        );
    }

    /// 単数形の接頭辞（`step_`）は取り除かない
    ///
    /// 規約上テーブル名は常に複数形のため、単数形に一致させる必要はない。
    /// むしろ単数形を許容すると、別テーブル `step_notes` の制約名
    /// `step_notes_step_id_fkey` がエンティティ `step` の接頭辞として誤って一致し、
    /// 誤ったエンティティ名（`notes_step`）を導いてしまう。
    #[test]
    fn test_strip_table_prefix_does_not_match_singular_prefix() {
        assert_eq!(strip_table_prefix_opt("step_notes_step_id", "step"), None);
        assert_eq!(
            strip_table_prefix("step_notes_step_id", "step"),
            "step_notes_step_id"
        );
    }

    #[test]
    fn test_strip_table_prefix_returns_none_for_other_table() {
        assert_eq!(strip_table_prefix_opt("trials_project_id", "project"), None);
    }

    #[test]
    fn test_strip_prefixes_removes_index_prefix() {
        assert_eq!(
            strip_prefixes("idx_projects_name", &INDEX_PREFIXES),
            "projects_name"
        );
        assert_eq!(
            strip_prefixes("projects_pkey", &INDEX_PREFIXES),
            "projects_pkey"
        );
    }

    #[test]
    fn test_strip_suffixes_removes_unique_suffix() {
        assert_eq!(
            strip_suffixes("steps_trial_id_position_key", &UNIQUE_SUFFIXES),
            "steps_trial_id_position"
        );
        assert_eq!(
            strip_suffixes("idx_projects_name", &UNIQUE_SUFFIXES),
            "idx_projects_name"
        );
    }

    /// `backend/migrations/` に実在する制約名・インデックス名が規約どおり解決できること
    #[test]
    fn test_real_constraint_names_follow_conventions() {
        // 主キー: {table}_pkey
        for (name, entity) in [
            ("projects_pkey", "project"),
            ("trials_pkey", "trial"),
            ("steps_pkey", "step"),
            ("parameters_pkey", "parameter"),
        ] {
            assert!(name.ends_with(PRIMARY_KEY_SUFFIX));
            assert_eq!(
                strip_table_prefix_opt(name, entity),
                Some(PRIMARY_KEY_SUFFIX.trim_start_matches('_'))
            );
        }

        // 一意制約: {table}_{columns}_key
        let field = strip_suffixes("steps_trial_id_position_key", &UNIQUE_SUFFIXES);
        assert_eq!(strip_table_prefix(field, "step"), "trial_id_position");

        // 一意インデックス: idx_{table}_{columns}
        let field = strip_prefixes("idx_projects_name", &INDEX_PREFIXES);
        assert_eq!(strip_table_prefix(field, "project"), "name");

        // 非一意インデックス: idx_{table}_{columns}
        let field = strip_prefixes("idx_trials_project_id", &INDEX_PREFIXES);
        assert_eq!(strip_table_prefix(field, "trial"), "project_id");
        let field = strip_prefixes("idx_parameters_step_id", &INDEX_PREFIXES);
        assert_eq!(strip_table_prefix(field, "parameter"), "step_id");

        // 外部キー: {table}_{column}_fkey（{table} は参照する側）
        for (name, referencing, referenced_column) in [
            ("trials_project_id_fkey", "trial", "project_id"),
            ("steps_trial_id_fkey", "step", "trial_id"),
            ("parameters_step_id_fkey", "parameter", "step_id"),
        ] {
            let body = name.strip_suffix(FOREIGN_KEY_SUFFIX).unwrap();
            assert_eq!(
                strip_table_prefix_opt(body, referencing),
                Some(referenced_column)
            );
        }
    }
}
