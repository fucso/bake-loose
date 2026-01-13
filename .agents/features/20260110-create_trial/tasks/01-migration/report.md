# Task Report: DB マイグレーション

> 実行日時: 2026-01-10

## 実行結果

### 1. マイグレーション実行

```
$ docker compose exec backend bash -c "sqlx migrate run"
Applied 20260110100001/migrate create trials (35.214458ms)
Applied 20260110100002/migrate create steps (4.720875ms)
Applied 20260110100003/migrate create parameters (6.199292ms)
```

**結果:** 3つのマイグレーションが正常に適用された

---

### 2. 作成されたスキーマ

#### trials テーブル

```
                                      Table "public.trials"
   Column   |           Type           | Collation | Nullable |             Default
------------+--------------------------+-----------+----------+----------------------------------
 id         | uuid                     |           | not null |
 project_id | uuid                     |           | not null |
 status     | character varying(20)    |           | not null | 'in_progress'::character varying
 memo       | text                     |           |          |
 created_at | timestamp with time zone |           | not null | now()
 updated_at | timestamp with time zone |           | not null | now()
Indexes:
    "trials_pkey" PRIMARY KEY, btree (id)
    "idx_trials_project_id" btree (project_id)
Foreign-key constraints:
    "trials_project_id_fkey" FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
Referenced by:
    TABLE "steps" CONSTRAINT "steps_trial_id_fkey" FOREIGN KEY (trial_id) REFERENCES trials(id) ON DELETE CASCADE
```

#### steps テーブル

```
                          Table "public.steps"
   Column   |           Type           | Collation | Nullable | Default
------------+--------------------------+-----------+----------+---------
 id         | uuid                     |           | not null |
 trial_id   | uuid                     |           | not null |
 name       | character varying(100)   |           |          |
 position   | integer                  |           | not null |
 started_at | timestamp with time zone |           |          |
 created_at | timestamp with time zone |           | not null | now()
 updated_at | timestamp with time zone |           | not null | now()
Indexes:
    "steps_pkey" PRIMARY KEY, btree (id)
    "idx_steps_trial_id" btree (trial_id)
    "steps_trial_id_position_key" UNIQUE CONSTRAINT, btree (trial_id, "position")
Foreign-key constraints:
    "steps_trial_id_fkey" FOREIGN KEY (trial_id) REFERENCES trials(id) ON DELETE CASCADE
Referenced by:
    TABLE "parameters" CONSTRAINT "parameters_step_id_fkey" FOREIGN KEY (step_id) REFERENCES steps(id) ON DELETE CASCADE
```

#### parameters テーブル

```
                           Table "public.parameters"
    Column    |           Type           | Collation | Nullable | Default
--------------+--------------------------+-----------+----------+---------
 id           | uuid                     |           | not null |
 step_id      | uuid                     |           | not null |
 content_type | character varying(20)    |           | not null |
 content      | jsonb                    |           | not null |
 created_at   | timestamp with time zone |           | not null | now()
 updated_at   | timestamp with time zone |           | not null | now()
Indexes:
    "parameters_pkey" PRIMARY KEY, btree (id)
    "idx_parameters_step_id" btree (step_id)
Foreign-key constraints:
    "parameters_step_id_fkey" FOREIGN KEY (step_id) REFERENCES steps(id) ON DELETE CASCADE
```

---

### 3. サンプルデータ登録結果

#### trials (2件)

| id | project_id | status | memo |
|----|------------|--------|------|
| aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa | 11111111-... | in_progress | 初めてのバゲット |
| aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab | 11111111-... | completed | 2回目の試行 |

#### steps (4件)

| id | trial_id | name | position | started_at |
|----|----------|------|----------|------------|
| bbbbbbbb-...-01 | aaaaaaaa-...-aa | こね | 0 | 2026-01-10 01:00:00+00 |
| bbbbbbbb-...-02 | aaaaaaaa-...-aa | 一次発酵 | 1 | 2026-01-10 01:15:00+00 |
| bbbbbbbb-...-03 | aaaaaaaa-...-aa | 焼成 | 2 | NULL |
| bbbbbbbb-...-04 | aaaaaaaa-...-ab | NULL | 0 | 2026-01-09 00:00:00+00 |

#### parameters (11件)

| step_id | content_type | content (概要) |
|---------|--------------|----------------|
| ...-01 (こね) | key_value | 粉: 300g |
| ...-01 (こね) | key_value | 水: 195g |
| ...-01 (こね) | key_value | 水温: 28℃ |
| ...-01 (こね) | duration_range | 15分 |
| ...-02 (一次発酵) | duration_range | 90分 (室温25度) |
| ...-02 (一次発酵) | text | パンチは30分後と60分後に実施 |
| ...-03 (焼成) | key_value | 初期温度: 250℃ |
| ...-03 (焼成) | duration_range | 25分 |
| ...-03 (焼成) | time_point | 10分後: 温度を230度に変更 |
| ...-03 (焼成) | time_point | 15分後: 天板の上下を入れ替え |
| ...-04 | text | シンプルな記録 |

---

### 4. 制約テスト

#### UNIQUE 制約 (trial_id, position)

```sql
-- 同一 trial_id と position の組み合わせで登録を試行
INSERT INTO steps (id, trial_id, name, position, started_at)
VALUES ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'test', 0, NULL);
```

**結果:**
```
ERROR:  duplicate key value violates unique constraint "steps_trial_id_position_key"
DETAIL:  Key (trial_id, "position")=(aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa, 0) already exists.
```

#### CASCADE 削除

```sql
-- Trial を削除して関連する Step と Parameter も削除されることを確認
DELETE FROM trials WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab';
SELECT COUNT(*) AS remaining_steps FROM steps WHERE trial_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab';
```

**結果:**
```
 remaining_steps
-----------------
               0
```

Trial 削除時に関連する Steps も CASCADE 削除された。

---

## 確認項目チェックリスト

- [x] `sqlx migrate run` が正常に完了する
- [x] trials, steps, parameters の各テーブルが作成されている
- [x] 外部キー制約が正しく動作する（親削除時に子も削除される）
- [x] UNIQUE 制約が正しく動作する（steps の trial_id + position）
- [x] サンプルデータが正常に登録できる

## 完了条件

- [x] 3 つのマイグレーションファイルが作成されている
- [x] `sqlx migrate run` が正常に完了する
- [x] 各テーブルのカラム・制約が設計通りになっている
