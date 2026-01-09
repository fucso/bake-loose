# Task Report: DBマイグレーション（projects テーブル）

> 実施日時: 2025-12-21 17:03
> 依存タスク: 02-infrastructure

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/migrations/20251221170317_create_projects.sql` | 新規 | projects テーブル作成マイグレーション |
| `backend/Dockerfile` | 修正 | sqlx-cli インストール追加 |

## ビルド・テスト結果

### cargo check

```
warning: function `create_pool` is never used
  --> src/infrastructure/database.rs:10:14
   |
10 | pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
   |              ^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `bake-loose` (bin "bake-loose") generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.22s
```

### sqlx migrate run

```
Applied 20251221170317/migrate create projects (2.217666ms)
```

## エビデンス

### テーブル構造

```
                        Table "public.projects"
   Column   |           Type           | Collation | Nullable | Default
------------+--------------------------+-----------+----------+---------
 id         | uuid                     |           | not null |
 name       | character varying(100)   |           | not null |
 created_at | timestamp with time zone |           | not null | now()
 updated_at | timestamp with time zone |           | not null | now()
Indexes:
    "projects_pkey" PRIMARY KEY, btree (id)
    "idx_projects_name" UNIQUE, btree (name)
```

### サンプルデータ投入結果

```sql
INSERT INTO projects (id, name) VALUES
  ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'ピザ生地研究'),
  ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'カンパーニュ'),
  ('c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'フォカッチャ')
ON CONFLICT (id) DO NOTHING;
-- INSERT 0 3
```

```
                  id                  |     name     |          created_at          |          updated_at
--------------------------------------+--------------+------------------------------+------------------------------
 a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11 | ピザ生地研究 | 2025-12-21 08:06:37.63319+00 | 2025-12-21 08:06:37.63319+00
 b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22 | カンパーニュ | 2025-12-21 08:06:37.63319+00 | 2025-12-21 08:06:37.63319+00
 c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33 | フォカッチャ | 2025-12-21 08:06:37.63319+00 | 2025-12-21 08:06:37.63319+00
(3 rows)
```

## 先送り事項

- [ ] `create_pool` 関数が未使用警告（09-integration で main.rs から呼び出し予定）

## 次タスクへの申し送り

- **sqlx-cli が利用可能**: `docker compose exec backend bash -c "sqlx migrate run"` でマイグレーション実行可能
- **サンプルデータ投入済み**: 3件のプロジェクトがDBに存在（開発・テスト用）
- **name カラムにユニーク制約**: 同名のプロジェクトは作成不可（`idx_projects_name`）
