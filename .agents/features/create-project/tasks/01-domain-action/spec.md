# Task: Domain アクション実装

> Feature: [create-project](../../spec.md)
> 依存: なし

## 目的
プロジェクト新規作成のためのドメインアクション `create_project` を実装する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/domain/actions/project.rs` | 新規 | project アクションモジュール定義 |
| `src/domain/actions/project/create_project.rs` | 新規 | create_project アクション |
| `src/domain/actions.rs` | 修正 | project モジュールを追加 |

---

## 設計詳細

### アクション構造

validate / execute / run パターンで実装する。

- **Command**: 入力データを保持する構造体
  - `name: String` - プロジェクト名

- **Error**: バリデーションエラーの種類を定義する enum
  - `EmptyName` - 名前が空の場合
  - `NameTooLong { max: usize, actual: usize }` - 名前が長すぎる場合

- **validate**: Command をバリデーションする関数
  - 名前が空でないことを確認
  - 名前が100文字以内であることを確認（DB制約に合わせる）

- **execute**: 新しい Project を生成する関数
  - UUID を生成し、新しい Project インスタンスを返す

- **run**: validate + execute を組み合わせた関数

### 名前の長さ制限

既存の DB マイグレーション（`VARCHAR(100)`）に合わせて、最大100文字とする。

### ID 生成

`uuid::Uuid::new_v4()` を使用して新規 ID を生成する。ドメイン層は現在 `uuid` クレートに依存しているため、追加依存なし。

---

## テストケース

ファイル: `src/domain/actions/project/create_project.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_run_creates_project_with_valid_name` | 有効な名前で Project が生成される |
| `test_run_creates_project_with_max_length_name` | 100文字ちょうどの名前で Project が生成される |
| `test_execute_generates_unique_id` | execute が UUID を生成する |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_validate_returns_error_for_empty_name` | 空文字列で `EmptyName` エラー |
| `test_validate_returns_error_for_whitespace_only_name` | 空白のみで `EmptyName` エラー |
| `test_validate_returns_error_for_too_long_name` | 101文字以上で `NameTooLong` エラー |

---

## 完了条件

- [ ] `create_project.rs` が validate / execute / run パターンで実装されている
- [ ] 空の名前でエラーが返る
- [ ] 100文字を超える名前でエラーが返る
- [ ] 正常な入力で新しい Project が生成される
- [ ] 上記テストケースがすべて実装されている
- [ ] `cargo test` が通る
