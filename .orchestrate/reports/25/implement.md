# Sub-Issue #25 実装レポート

## 実装概要

`Parameter`/`ParameterContent`/`ParameterValue`/`DurationValue`/`DurationUnit` をドメインモデルとして新規定義し、
`Step` に `parameters` フィールドと `add_parameter`/`remove_parameter` を配線した。
あわせて `add_step`/`update_step`（後続サブ Issue #29/#30）から共通利用できる `parameter_validator` を追加し、
過去 PR #18 のレビュー指摘（`DurationUnit` enum 化、`note` 必須化、`DurationValue` 非負検証、
`Quantity.unit` 空文字チェック）にすべて対応した設計とした。

- `ParameterContent` は KeyValue / Duration / TimeMarker / Text の4バリアントを持つ enum（`#[serde(tag = "type")]`）
- `ParameterValue`（KeyValue の value 部分）は Text / Quantity の2バリアント
- `DurationValue { value: f64, unit: DurationUnit }`、`DurationUnit` は Day/Hour/Minute/Second の enum
- `Duration`/`TimeMarker` の `note` は `String`（`Option` ではない）で必須を型レベルで保証
- `parameter_validator::validate` で `DurationValue` の非負検証と `Quantity.unit` の空文字検証を実施

## 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `backend/src/domain/models/parameter.rs` | 新規追加。`Parameter`/`ParameterId`/`ParameterContent`/`ParameterValue`/`DurationValue`/`DurationUnit` を定義 |
| `backend/src/domain/models/step.rs` | `parameters: Vec<Parameter>` フィールド、`parameters()`/`add_parameter()`/`remove_parameter()` を追加。`new()`/`from_raw()` の署名変更。`Eq` derive を削除（`f64` を含むため） |
| `backend/src/domain/models/trial.rs` | `Step` の `Eq` derive 削除に追従して `Trial` の `Eq` derive も削除 |
| `backend/src/domain/models.rs` | `pub mod parameter;` を追加 |
| `backend/src/domain/validators/trial/parameter_validator.rs` | 新規追加。`DurationValue` 非負検証・`Quantity.unit` 空文字検証を行う `validate()` |
| `backend/src/domain/validators/trial.rs` | `pub mod parameter_validator;` を追加 |
| `backend/src/domain/validators/trial/step_status_validator.rs` | `Step::from_raw` の呼び出し箇所を新しい引数（`parameters`）に追従修正 |

## 品質チェック結果

```
=== 品質チェック ===
環境名: iddue/25
パス: .worktree/iddue/25

変更ファイル: backend/src/domain/models.rs
backend/src/domain/models/step.rs
backend/src/domain/models/trial.rs
backend/src/domain/validators/trial.rs
backend/src/domain/validators/trial/step_status_validator.rs
--- cargo fmt --check ---
--- cargo clippy --all-targets -- -D warnings ---
    Checking bake-loose v0.1.0 (/worktrees/iddue/25/backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.22s
--- cargo test ---
   Compiling bake-loose v0.1.0 (/worktrees/iddue/25/backend)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.53s
     Running unittests src/lib.rs

running 59 tests
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s

     Running unittests src/main.rs

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/graphql.rs

running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

   Doc-tests bake_loose
running 1 test
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## レビュー結果

✅ レビュー OK（全観点 CRITICAL・WARNING なし）

補足（非ブロッキング INFO、2件）:
- `ParameterValue::Quantity.amount` に非負検証はない（Issue の完了条件には含まれない）
- `Duration`/`TimeMarker` の `note` は型として必須（`Option` ではない）だが、空文字 `""` は `validate()` を通過する

## コミット情報

- Branch: iddue/25
- Commit: 610f24a
- Message: `[Issue#25] domain-model: Parameter モデルとバリデーション`

## 引き継ぎ事項

- 後続サブ Issue #29（`add_step`）/ #30（`update_step`）は `domain::validators::trial::parameter_validator::validate` を呼び出すことで、
  Parameter 追加時のバリデーション（`DurationValue` 非負・`Quantity.unit` 空文字）を実施できる。
- `Parameter::from_raw` はリポジトリ層専用（`domain.md` の規約通り）。Action 層では `Parameter::new(step_id, content)` を使用すること。
- `Step`/`Trial` から `Eq` derive を削除した（`f64` を含む `DurationValue`/`ParameterValue::Quantity` が `Eq` を実装できないため）。今後 `Step`/`Trial` を `HashSet`/`HashMap` のキーにする実装は不可なので注意。
- `parameters` テーブルは `content` を JSONB 1カラムに集約するスキーマ（migration `20260730000002_create_parameters.sql`、Issue #23 で追加済み）。リポジトリ実装時は `ParameterContent` を serde でシリアライズしてそのまま格納する想定。

## ステータス

completed

## Metadata

- **開始:** 2026-07-30 22:20:20 JST
- **終了:** 2026-07-30 22:33:04 JST
- **実行時間:** 12分39秒
- **消費トークン:** output 83407 / cache_read 14921798 / cache_write 273296
- **Claude ログ:** /Users/sohosoki/.claude/projects/-Users-sohosoki-dev-fucso-bake-loose/cb02e4ac-640b-49a5-b54b-f40da2b822b7.jsonl
