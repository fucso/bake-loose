//! DB スキーマ命名規約の検証テスト
//!
//! `backend/migrations/` の SQL が `.claude/rules/backend/repository.md` の
//! 「スキーマ命名規約」に従っているかを、**実際に適用したスキーマ** を走査して検証する。
//!
//! # なぜ SQL の静的解析ではなくスキーマ走査なのか
//!
//! 本プロジェクトの規約は「主キー・一意制約・外部キーは PostgreSQL が付与するデフォルト名を
//! そのまま使う」である。デフォルト名は SQL 文中に現れず PostgreSQL が内部で決めるため、
//! SQL ファイルを静的に読むだけでは実際の制約名を知ることができない。
//! マイグレーション適用後のカタログ（`pg_constraint` / `pg_index`）を読めば、
//! 暗黙のデフォルト名も明示的な命名も同じ土俵で検証できる。
//!
//! # 実 DB を汚さない
//!
//! `#[sqlx::test]` は呼び出しごとに使い捨てのデータベースを生成し、
//! `migrations = "./migrations"` の全マイグレーションを適用してからテスト関数に渡す。
//! 開発者の永続的なローカル DB（`compose.yaml` の `db` サービス）には一切触れないため、
//! 「`cargo test` で検証 → pass 後に `sqlx migrate run` で実 DB へ適用」の順序を守れば、
//! 規約違反のマイグレーションが実 DB に適用される前に検出できる。
//! 運用手順は `AGENTS.md` 4 章を参照。
//!
//! # 検査対象
//!
//! - 主キー制約名: `{table}_pkey`
//! - 一意制約名: `{table}_{columns}_key`
//! - 外部キー制約名: `{table}_{columns}_fkey`（`{table}` は参照する側）
//! - インデックス名: `idx_{table}_{columns}`
//!
//! CHECK 制約は規約が定義されていない（`naming_conventions.rs` に対応する定数がない）ため
//! 検査対象に含めない。規約を定める場合はここに検査を追加すること。
//!
//! 例外リストは設けず、`public` スキーマのテーブルをすべて検査する。
//! ただし `_sqlx_migrations` は sqlx がマイグレーション管理のために自動生成するテーブルであり、
//! 本プロジェクトのマイグレーションが作るものではないため除外する。

use bake_loose::repository::naming_conventions::{
    FOREIGN_KEY_SUFFIX, INDEX_PREFIX, PRIMARY_KEY_SUFFIX, UNIQUE_CONSTRAINT_SUFFIX,
};
use sqlx::{FromRow, PgPool};

/// sqlx がマイグレーション管理のために自動生成するテーブル
const SQLX_MIGRATIONS_TABLE: &str = "_sqlx_migrations";

/// PostgreSQL の識別子の最大長（`NAMEDATALEN - 1` バイト）
///
/// 名前がこの長さを超えた場合の扱いは、名前を **誰が付けたか** で異なる。
/// 検証側も同じ規則で期待値を組み立てる（`truncate_identifier` と
/// `default_constraint_name` を参照）。
const MAX_IDENTIFIER_LENGTH: usize = 63;

/// `public` スキーマの主キー・一意制約・外部キーを取得するクエリ
///
/// `conkey` は制約を構成するカラムの属性番号の配列で、並び順が
/// PostgreSQL のデフォルト制約名に現れるカラムの順序と一致する。
const CONSTRAINTS_QUERY: &str = r#"
SELECT
    rel.relname::text AS table_name,
    con.conname::text AS constraint_name,
    con.contype::text AS constraint_type,
    ARRAY(
        SELECT att.attname::text
        FROM unnest(con.conkey) WITH ORDINALITY AS k(attnum, ord)
        JOIN pg_attribute att
            ON att.attrelid = con.conrelid AND att.attnum = k.attnum
        ORDER BY k.ord
    ) AS column_names
FROM pg_constraint con
JOIN pg_class rel ON rel.oid = con.conrelid
JOIN pg_namespace nsp ON nsp.oid = rel.relnamespace
WHERE nsp.nspname = 'public'
  AND con.contype IN ('p', 'u', 'f')
  AND rel.relname <> $1
ORDER BY rel.relname, con.conname
"#;

/// `public` スキーマの「制約に紐づかない」インデックスを取得するクエリ
///
/// 主キー・一意制約が内部的に作るインデックスは制約名と同名であり、
/// 制約側の検査と二重になるため `pg_constraint.conindid` との結合で除外する。
/// ただし `conindid` は**外部キー制約**にも設定され、その場合は参照**先**テーブルの
/// インデックスを指す。除外条件を `contype` で絞らないと、外部キーから参照された
/// 一意インデックス（本プロジェクトが `CREATE UNIQUE INDEX` で作るもの）が
/// 検査対象から静かに外れてしまうため、`p` / `u` / `x` に限定する。
///
/// `indkey` は `INCLUDE` で指定した非キーカラムも含むが、インデックス名に現れるのはキーカラムのみ
/// のため、`indnkeyatts` 件目までに限定する。`indkey` は添字が 0 始まりの `int2vector` であり
/// 配列スライスの添字がずれるため、`WITH ORDINALITY` の連番（常に 1 始まり）で絞り込む。
const INDEXES_QUERY: &str = r#"
SELECT
    rel.relname::text AS table_name,
    idx_cls.relname::text AS index_name,
    ARRAY(
        SELECT att.attname::text
        FROM unnest(idx.indkey::smallint[]) WITH ORDINALITY AS k(attnum, ord)
        JOIN pg_attribute att
            ON att.attrelid = idx.indrelid AND att.attnum = k.attnum
        WHERE k.ord <= idx.indnkeyatts
        ORDER BY k.ord
    ) AS column_names,
    EXISTS (
        SELECT 1
        FROM unnest(idx.indkey::smallint[]) WITH ORDINALITY AS k(attnum, ord)
        WHERE k.ord <= idx.indnkeyatts
          AND k.attnum = 0
    ) AS is_expression_index
FROM pg_index idx
JOIN pg_class idx_cls ON idx_cls.oid = idx.indexrelid
JOIN pg_class rel ON rel.oid = idx.indrelid
JOIN pg_namespace nsp ON nsp.oid = rel.relnamespace
LEFT JOIN pg_constraint con
    ON con.conindid = idx.indexrelid
   AND con.contype IN ('p', 'u', 'x')
WHERE nsp.nspname = 'public'
  AND rel.relkind IN ('r', 'p')
  AND con.oid IS NULL
  AND rel.relname <> $1
ORDER BY rel.relname, idx_cls.relname
"#;

/// 主キー・一意制約・外部キーの 1 件
#[derive(Debug, FromRow)]
struct ConstraintRow {
    table_name: String,
    constraint_name: String,
    /// `pg_constraint.contype`（`p` = 主キー / `u` = 一意制約 / `f` = 外部キー）
    constraint_type: String,
    column_names: Vec<String>,
}

/// 制約に紐づかないインデックスの 1 件
#[derive(Debug, FromRow)]
struct IndexRow {
    table_name: String,
    index_name: String,
    column_names: Vec<String>,
    /// カラムではなく式に対するインデックスかどうか
    ///
    /// 式インデックスは規約上のカラム名を決められないため、接頭辞のみを検査する。
    is_expression_index: bool,
}

/// 指定バイト数以内に収まる最大の文字境界で切り詰める
///
/// 識別子は ASCII 想定だが、マルチバイト文字でもパニックしないよう文字境界まで戻す
/// （PostgreSQL の `pg_mbcliplen` に相当）。
fn clip_to_char_boundary(name: &str, max_bytes: usize) -> &str {
    if name.len() <= max_bytes {
        return name;
    }

    let mut end = max_bytes;
    while !name.is_char_boundary(end) {
        end -= 1;
    }
    &name[..end]
}

/// SQL に明示的に書かれた識別子を PostgreSQL と同じ規則で切り詰める
///
/// `CREATE INDEX idx_...` のように開発者が明示的に書いた識別子は、
/// 63 バイトを超えると PostgreSQL が末尾を単純に切り捨てる（`NOTICE: identifier will be
/// truncated` が出る）。制約のデフォルト名とは切り詰め規則が異なる点に注意。
fn truncate_identifier(name: &str) -> &str {
    clip_to_char_boundary(name, MAX_IDENTIFIER_LENGTH)
}

/// PostgreSQL が制約・インデックスに付けるデフォルト名を再現する
///
/// PostgreSQL の `makeObjectName()`（`ChooseConstraintName` / `ChooseIndexName` が使う）の移植。
/// 単純な末尾切り捨てでは**ない**点が重要で、`_pkey` / `_key` / `_fkey` といったラベルは必ず末尾に残し、
/// テーブル名とカラム名の側を——長い方を優先して 1 バイトずつ——削って 63 バイトに収める。
///
/// - `table`: テーブル名（`name1`）
/// - `columns`: 制約を構成するカラム名を `_` で連結したもの（`name2`）。主キーは `None`
/// - `label`: `pkey` / `key` / `fkey`（先頭の `_` を含まない）
///
/// 同名の制約が既に存在する場合、PostgreSQL は末尾に連番を付ける（`..._key1`）。
/// その場合は規約どおりの名前と一致せず違反として報告されるが、
/// 名前から対象を復元できない曖昧な状態であり、報告されるのが望ましい。
fn default_constraint_name(table: &str, columns: Option<&str>, label: &str) -> String {
    // ラベルとその直前の `_`
    let mut overhead = label.len() + 1;

    let mut name1_bytes = table.len();
    let mut name2_bytes = match columns {
        Some(columns) => {
            // カラム名との間の `_`
            overhead += 1;
            columns.len()
        }
        None => 0,
    };

    let available = MAX_IDENTIFIER_LENGTH - overhead;

    // 溢れる分は長い方から 1 バイトずつ削る（makeObjectName と同じ挙動）
    while name1_bytes + name2_bytes > available {
        if name1_bytes > name2_bytes {
            name1_bytes -= 1;
        } else {
            name2_bytes -= 1;
        }
    }

    let mut name = String::with_capacity(MAX_IDENTIFIER_LENGTH);
    name.push_str(clip_to_char_boundary(table, name1_bytes));
    if let Some(columns) = columns {
        name.push('_');
        name.push_str(clip_to_char_boundary(columns, name2_bytes));
    }
    name.push('_');
    name.push_str(label);
    name
}

/// 実際の名前が規約どおりの名前と一致するか
///
/// 明示的に書かれた識別子（インデックス名）用。63 バイト超は単純切り捨てで比較する。
/// `truncate_identifier` は 63 バイト以下の名前をそのまま返すため、切り詰めの有無を場合分けしなくてよい。
fn matches_convention(actual: &str, expected: &str) -> bool {
    actual == truncate_identifier(expected)
}

/// 規約違反を表すメッセージを組み立てる
fn violation(kind: &str, table: &str, actual: &str, expected: &str) -> String {
    format!("{table}: {kind} `{actual}` は規約では `{expected}` であるべきです")
}

/// 制約 1 件を検査し、違反していればメッセージを返す
fn check_constraint(row: &ConstraintRow) -> Option<String> {
    let table = &row.table_name;
    let columns = row.column_names.join("_");

    // 規約上これらは PostgreSQL のデフォルト名をそのまま使うため、
    // 期待値も PostgreSQL と同じ規則（ラベルを残して名前側を削る）で組み立てる。
    // 主キーの名前はカラムに依存しない（`makeObjectName(table, NULL, "pkey")`）。
    let (kind, expected) = match row.constraint_type.as_str() {
        "p" => (
            "主キー制約名",
            default_constraint_name(table, None, PRIMARY_KEY_SUFFIX.trim_start_matches('_')),
        ),
        "u" => (
            "一意制約名",
            default_constraint_name(
                table,
                Some(&columns),
                UNIQUE_CONSTRAINT_SUFFIX.trim_start_matches('_'),
            ),
        ),
        "f" => (
            "外部キー制約名",
            default_constraint_name(
                table,
                Some(&columns),
                FOREIGN_KEY_SUFFIX.trim_start_matches('_'),
            ),
        ),
        other => unreachable!("検査対象外の制約種別が取得されました: {other}"),
    };

    if row.constraint_name == expected {
        None
    } else {
        Some(violation(kind, table, &row.constraint_name, &expected))
    }
}

/// インデックス 1 件を検査し、違反していればメッセージを返す
fn check_index(row: &IndexRow) -> Option<String> {
    let table = &row.table_name;

    // 式インデックスはカラム名が定まらないため、`idx_{table}_` までを検査する
    if row.is_expression_index {
        let expected_prefix = format!("{INDEX_PREFIX}{table}_");
        return if row.index_name.starts_with(&expected_prefix) {
            None
        } else {
            Some(violation(
                "インデックス名",
                table,
                &row.index_name,
                &format!("{expected_prefix}{{式の内容を表す名前}}"),
            ))
        };
    }

    let columns = row.column_names.join("_");
    let expected = format!("{INDEX_PREFIX}{table}_{columns}");

    if matches_convention(&row.index_name, &expected) {
        None
    } else {
        Some(violation(
            "インデックス名",
            table,
            &row.index_name,
            &expected,
        ))
    }
}

/// 適用済みスキーマを走査して規約違反を収集する
async fn collect_violations(pool: &PgPool) -> Vec<String> {
    let constraints: Vec<ConstraintRow> = sqlx::query_as(CONSTRAINTS_QUERY)
        .bind(SQLX_MIGRATIONS_TABLE)
        .fetch_all(pool)
        .await
        .expect("制約一覧の取得に失敗しました");

    let indexes: Vec<IndexRow> = sqlx::query_as(INDEXES_QUERY)
        .bind(SQLX_MIGRATIONS_TABLE)
        .fetch_all(pool)
        .await
        .expect("インデックス一覧の取得に失敗しました");

    constraints
        .iter()
        .filter_map(check_constraint)
        .chain(indexes.iter().filter_map(check_index))
        .collect()
}

/// 適用済みの全マイグレーションが命名規約に従っていること
#[sqlx::test(migrations = "./migrations")]
async fn test_migrations_follow_naming_conventions(pool: PgPool) {
    // クエリが何も返さないと検査が素通りしてしまうため、検査対象の存在を先に確かめる
    let constraint_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM pg_constraint con
        JOIN pg_class rel ON rel.oid = con.conrelid
        JOIN pg_namespace nsp ON nsp.oid = rel.relnamespace
        WHERE nsp.nspname = 'public'
          AND con.contype IN ('p', 'u', 'f')
          AND rel.relname <> $1
        "#,
    )
    .bind(SQLX_MIGRATIONS_TABLE)
    .fetch_one(&pool)
    .await
    .expect("制約数の取得に失敗しました");

    assert!(
        constraint_count > 0,
        "public スキーマに検査対象の制約が 1 件もありません。\
         マイグレーションが適用されていない可能性があります"
    );

    let violations = collect_violations(&pool).await;

    assert!(
        violations.is_empty(),
        "スキーマ命名規約に違反しています（規約: .claude/rules/backend/repository.md）:\n{}",
        violations.join("\n")
    );
}

/// 規約違反のスキーマを実際に検出できること
///
/// 検査ロジックが常に空の結果を返すだけの「素通り」になっていないことを保証する。
/// 使い捨て DB 上に意図的に規約違反のテーブルを作って検出結果を確認する。
#[sqlx::test(migrations = "./migrations")]
async fn test_detects_naming_violations(pool: PgPool) {
    /// 使い捨て DB 上にだけ作る検証用テーブル
    const PROBE_TABLE: &str = "naming_probes";

    sqlx::query(
        r#"
        CREATE TABLE naming_probes (
            id UUID PRIMARY KEY,
            probe_id UUID NOT NULL,
            CONSTRAINT naming_probes_unique_probe_id UNIQUE (probe_id)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("検証用テーブルの作成に失敗しました");

    sqlx::query(
        r#"
        ALTER TABLE naming_probes
            ADD CONSTRAINT fk_naming_probes_probe_id
            FOREIGN KEY (probe_id) REFERENCES naming_probes(id)
        "#,
    )
    .execute(&pool)
    .await
    .expect("検証用外部キーの作成に失敗しました");

    sqlx::query("CREATE INDEX naming_probes_probe_id_index ON naming_probes(probe_id)")
        .execute(&pool)
        .await
        .expect("検証用インデックスの作成に失敗しました");

    // 検証用テーブル以外の違反は本テストの関心事ではない
    // （マイグレーション自体の違反は test_migrations_follow_naming_conventions が検出する）
    let mut detected: Vec<String> = collect_violations(&pool)
        .await
        .into_iter()
        .filter(|v| v.starts_with(&format!("{PROBE_TABLE}: ")))
        .collect();
    detected.sort();

    // 主キー `naming_probes_pkey` は規約どおりのため検出されない
    let mut expected = vec![
        "naming_probes: 一意制約名 `naming_probes_unique_probe_id` は規約では \
         `naming_probes_probe_id_key` であるべきです"
            .to_string(),
        "naming_probes: 外部キー制約名 `fk_naming_probes_probe_id` は規約では \
         `naming_probes_probe_id_fkey` であるべきです"
            .to_string(),
        "naming_probes: インデックス名 `naming_probes_probe_id_index` は規約では \
         `idx_naming_probes_probe_id` であるべきです"
            .to_string(),
    ];
    expected.sort();

    assert_eq!(detected, expected);
}

/// 外部キーから参照されている一意インデックスも検査対象から外れないこと
///
/// `pg_constraint.conindid` は外部キー制約にも設定され、参照**先**テーブルのインデックスを指す。
/// 除外条件を `contype` で絞らないと、外部キーに参照された途端に一意インデックスの名前検査が
/// 静かに無効化される（偽陰性）。本プロジェクトは一意性を `CREATE UNIQUE INDEX` で表現するため、
/// この取りこぼしが実際に起こりうる。
#[sqlx::test(migrations = "./migrations")]
async fn test_checks_unique_index_referenced_by_foreign_key(pool: PgPool) {
    // 規約違反の名前（`idx_` 接頭辞なし）を持つ一意インデックスを作る
    sqlx::query("CREATE TABLE fk_probe_parents (id UUID PRIMARY KEY, code VARCHAR(50) NOT NULL)")
        .execute(&pool)
        .await
        .expect("検証用の参照先テーブルの作成に失敗しました");

    sqlx::query("CREATE UNIQUE INDEX fk_probe_parents_code_unique ON fk_probe_parents(code)")
        .execute(&pool)
        .await
        .expect("検証用の一意インデックスの作成に失敗しました");

    // この外部キーの conindid が上の一意インデックスを指す
    sqlx::query(
        r#"
        CREATE TABLE fk_probe_children (
            id UUID PRIMARY KEY,
            code VARCHAR(50) NOT NULL REFERENCES fk_probe_parents(code)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("検証用の参照元テーブルの作成に失敗しました");

    let detected: Vec<String> = collect_violations(&pool)
        .await
        .into_iter()
        .filter(|v| v.starts_with("fk_probe_parents: "))
        .collect();

    assert_eq!(
        detected,
        vec![
            "fk_probe_parents: インデックス名 `fk_probe_parents_code_unique` は規約では \
             `idx_fk_probe_parents_code` であるべきです"
                .to_string()
        ]
    );
}

/// PostgreSQL が切り詰めたデフォルト制約名を違反と誤判定しないこと
///
/// `default_constraint_name` は PostgreSQL の `makeObjectName()` の移植であり、
/// 移植が実物とずれていると「規約どおりに書いたのに違反と報告される」偽陽性を生む。
/// 63 バイトを超える名前になるテーブルを使い捨て DB 上に作り、
/// PostgreSQL が実際に付けた名前と期待値が一致することを突き合わせて確認する。
#[sqlx::test(migrations = "./migrations")]
async fn test_accepts_postgres_truncated_default_names(pool: PgPool) {
    let table = format!("long_{}", "t".repeat(40));
    let col_a = format!("col_a_{}", "u".repeat(30));
    let col_b = format!("col_b_{}", "v".repeat(30));

    // 制約名を明示せず、PostgreSQL にデフォルト名を付けさせる
    sqlx::query(&format!(
        "CREATE TABLE {table} (
             id UUID PRIMARY KEY,
             {col_a} UUID NOT NULL REFERENCES {table}(id),
             {col_b} UUID NOT NULL,
             UNIQUE ({col_a}, {col_b})
         )"
    ))
    .execute(&pool)
    .await
    .expect("検証用テーブルの作成に失敗しました");

    let names: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT con.conname::text
        FROM pg_constraint con
        JOIN pg_class rel ON rel.oid = con.conrelid
        WHERE rel.relname = $1
        "#,
    )
    .bind(&table)
    .fetch_all(&pool)
    .await
    .expect("検証用制約名の取得に失敗しました");

    // 切り詰めが起きていなければ、この検証は何も確かめていないことになる
    assert!(
        names.iter().any(|n| n.len() == MAX_IDENTIFIER_LENGTH),
        "63 バイトへの切り詰めが発生していないため検証になっていません: {names:?}"
    );

    let detected: Vec<String> = collect_violations(&pool)
        .await
        .into_iter()
        .filter(|v| v.starts_with(&format!("{table}: ")))
        .collect();

    assert!(
        detected.is_empty(),
        "PostgreSQL のデフォルト名を違反と誤判定しました:\n{}",
        detected.join("\n")
    );
}

/// 明示的に書いたインデックス名は 63 バイトで単純に切り捨てて比較する
#[test]
fn test_matches_convention_allows_truncated_identifier() {
    let expected = format!("{INDEX_PREFIX}{}_some_long_column", "a".repeat(60));
    assert!(expected.len() > MAX_IDENTIFIER_LENGTH);

    // PostgreSQL は 63 バイトに切り詰めた名前で登録するため、実際の名前はこれになる
    let truncated = &expected[..MAX_IDENTIFIER_LENGTH];
    assert!(matches_convention(truncated, &expected));

    assert!(!matches_convention("idx_totally_different", &expected));

    // 63 バイト以下なら切り詰めは起きず、そのまま一致する
    let short = format!("{INDEX_PREFIX}projects_name");
    assert!(matches_convention(&short, &short));
}

/// 63 バイト以下の名前はそのまま扱う
#[test]
fn test_truncate_identifier_keeps_short_name() {
    assert_eq!(
        truncate_identifier("idx_projects_name"),
        "idx_projects_name"
    );
    assert_eq!(truncate_identifier(&"a".repeat(63)), "a".repeat(63));
    assert_eq!(truncate_identifier(&"a".repeat(64)).len(), 63);
}

/// 63 バイトに収まる制約名はそのまま組み立てられる
#[test]
fn test_default_constraint_name_builds_postgres_default() {
    assert_eq!(
        default_constraint_name("projects", None, "pkey"),
        "projects_pkey"
    );
    assert_eq!(
        default_constraint_name("steps", Some("trial_id_position"), "key"),
        "steps_trial_id_position_key"
    );
    assert_eq!(
        default_constraint_name("trials", Some("project_id"), "fkey"),
        "trials_project_id_fkey"
    );
}

/// 63 バイトを超える制約名はラベルを残したまま名前側が削られる
///
/// 単純な末尾切り捨てでは `_key` が失われてしまうため、
/// PostgreSQL の `makeObjectName()` と同じく長い方から削る挙動を再現していることを確認する。
#[test]
fn test_default_constraint_name_truncates_preserving_label() {
    let table = "a".repeat(40);
    let columns = "b".repeat(40);

    let name = default_constraint_name(&table, Some(&columns), "key");

    assert_eq!(name.len(), MAX_IDENTIFIER_LENGTH);
    assert!(
        name.ends_with("_key"),
        "ラベルが末尾に残っていません: {name}"
    );

    // 利用可能な 58 バイト（63 - `_key` 4 - 区切り `_` 1）を均等に分け合う
    assert_eq!(name, format!("{}_{}_key", "a".repeat(29), "b".repeat(29)));
}

/// 長い方の名前が優先的に削られる
#[test]
fn test_default_constraint_name_truncates_longer_part_first() {
    let table = "a".repeat(10);
    let columns = "b".repeat(60);

    let name = default_constraint_name(&table, Some(&columns), "fkey");

    // `_fkey` 5 + 区切り `_` 1 = 6 バイトが overhead、残り 57 バイトを分け合う。
    // 短いテーブル名（10）はそのまま残り、カラム側だけが 47 バイトに削られる
    assert_eq!(name, format!("{}_{}_fkey", "a".repeat(10), "b".repeat(47)));
    assert_eq!(name.len(), MAX_IDENTIFIER_LENGTH);
}
