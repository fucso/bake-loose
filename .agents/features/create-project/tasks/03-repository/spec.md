# Task: Repository 実装

> Feature: [create-project](../../spec.md)
> 依存: 02-ports

## 目的
`PgProjectRepository` に `save`, `exists_by_name` の SQL 実装を追加する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/repository/project_repo.rs` | 修正 | `save`, `exists_by_name` の実装 |
| `src/use_case/test/mock_unit_of_work.rs` | 修正 | モックに `save`, `exists_by_name` を追加 |

---

## 設計詳細

### save 実装

UPSERT パターン（`INSERT ... ON CONFLICT DO UPDATE`）を使用する。

- **INSERT** 時: `id`, `name`, `created_at`, `updated_at` を設定
- **UPDATE** 時: `name`, `updated_at` のみ更新

SQL 例:
```sql
INSERT INTO projects (id, name, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    updated_at = NOW()
```

### exists_by_name 実装

`EXISTS` サブクエリを使用して効率的にチェックする。

SQL 例:
```sql
SELECT EXISTS(SELECT 1 FROM projects WHERE name = $1)
```

### エラー変換

SQLx のエラーを `RepositoryError` に変換する:
- 接続エラー → `RepositoryError::Connection`
- その他 → `RepositoryError::Internal { message: ... }`

### モック更新

`MockProjectRepository` にも対応するメソッドを追加する:
- `save`: 内部の HashMap に追加
- `exists_by_name`: 内部データから検索

---

## テストケース

ファイル: `src/repository/project_repo.rs` 内の `#[cfg(test)] mod tests`

### save メソッド

| テスト名 | 内容 |
|----------|------|
| `test_save_inserts_new_project` | 新規プロジェクトが INSERT される |
| `test_save_updates_existing_project` | 既存プロジェクトが UPDATE される |

### exists_by_name メソッド

| テスト名 | 内容 |
|----------|------|
| `test_exists_by_name_returns_true_when_exists` | 存在する名前で `true` が返る |
| `test_exists_by_name_returns_false_when_not_exists` | 存在しない名前で `false` が返る |

---

## 完了条件

- [ ] `save` が UPSERT パターンで実装されている
- [ ] `exists_by_name` が EXISTS クエリで実装されている
- [ ] エラーが適切に `RepositoryError` に変換されている
- [ ] `MockProjectRepository` が更新されている
- [ ] 上記テストケースがすべて実装されている
- [ ] `cargo test` が通る
