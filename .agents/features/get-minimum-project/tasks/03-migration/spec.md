# Task: DBマイグレーション（projects テーブル）

> Feature: [get-project](../../spec.md)
> 依存: 02-infrastructure

## 目的
projects テーブルを作成するマイグレーションファイルを作成し、実行する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/migrations/{timestamp}_create_projects.sql` | 新規 | projects テーブル作成 |

---

## 設計詳細

### テーブル定義

最小限のフィールドで開始する。将来的に description, goal, status などを追加予定。

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| `id` | `UUID` | PRIMARY KEY | プロジェクトID |
| `name` | `VARCHAR(100)` | NOT NULL | プロジェクト名 |
| `created_at` | `TIMESTAMP WITH TIME ZONE` | NOT NULL DEFAULT NOW() | 作成日時 |
| `updated_at` | `TIMESTAMP WITH TIME ZONE` | NOT NULL DEFAULT NOW() | 更新日時 |

### インデックス

- `idx_projects_name`: name カラムへの UNIQUE インデックス（名前の重複を防ぐ）

### サンプルデータ投入用 SQL

マイグレーション後、以下のようなSQLでサンプルデータを投入できる（参考として記載）:

```sql
INSERT INTO projects (id, name) VALUES
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'ピザ生地研究'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'カンパーニュ');
```

---

## 補足資料

| ファイル | 内容 |
|----------|------|
| [sample_data.sql](./sample_data.sql) | サンプルデータ投入用SQL（手動実行用） |

---

## 完了条件

- [ ] マイグレーションファイルが作成されている
- [ ] `sqlx migrate run` で正常にテーブルが作成される
- [ ] 手動でサンプルデータを投入できる
