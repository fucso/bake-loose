# Task: DB マイグレーション

> Feature: [create_trial](../../spec.md)
> 依存: なし

## 目的

Trial、Step、Parameter を格納するためのデータベーステーブルを作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/migrations/{timestamp}_create_trials.sql` | 新規 | trials テーブル作成 |
| `backend/migrations/{timestamp}_create_steps.sql` | 新規 | steps テーブル作成 |
| `backend/migrations/{timestamp}_create_parameters.sql` | 新規 | parameters テーブル作成 |

---

## 設計詳細

### trials テーブル

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | Trial の一意識別子 |
| project_id | UUID | NOT NULL, FK | 所属する Project |
| status | VARCHAR(20) | NOT NULL, DEFAULT 'in_progress' | ステータス |
| memo | TEXT | NULL | メモ |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**インデックス:**
- `idx_trials_project_id` on `project_id`

**外部キー:**
- `project_id` → `projects(id)` ON DELETE CASCADE

### steps テーブル

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | Step の一意識別子 |
| trial_id | UUID | NOT NULL, FK | 所属する Trial |
| name | VARCHAR(100) | NULL | ステップ名（任意） |
| position | INTEGER | NOT NULL | 順序 |
| started_at | TIMESTAMPTZ | NULL | 開始時刻 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**インデックス:**
- `idx_steps_trial_id` on `trial_id`

**制約:**
- `UNIQUE(trial_id, position)` - 同一 Trial 内で position は一意

**外部キー:**
- `trial_id` → `trials(id)` ON DELETE CASCADE

### parameters テーブル

Parameter は種類（KeyValue, Text, DurationRange, TimePoint）によって異なるデータを持つ。
JSONB を使用して柔軟に格納する。

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | Parameter の一意識別子 |
| step_id | UUID | NOT NULL, FK | 所属する Step |
| content_type | VARCHAR(20) | NOT NULL | パラメーター種別 |
| content | JSONB | NOT NULL | パラメーター内容 |
| position | INTEGER | NOT NULL | 順序（登録順） |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

**content_type の値:**
- `key_value`
- `text`
- `duration_range`
- `time_point`

**content の JSONB 構造:**

```json
// key_value (テキスト値)
{
  "key": "使用粉",
  "value": { "type": "text", "text": "リスドォル" }
}

// key_value (数量)
{
  "key": "粉",
  "value": { "type": "quantity", "amount": 300, "unit": "gram" }
}

// text
{
  "value": "発酵を冷蔵庫で行う"
}

// duration_range
{
  "duration_seconds": 900,
  "display_unit": "minute",
  "note": "室温25度"
}

// time_point
{
  "elapsed_seconds": 600,
  "display_unit": "minute",
  "note": "温度を230度に変更"
}
```

**インデックス:**
- `idx_parameters_step_id` on `step_id`

**外部キー:**
- `step_id` → `steps(id)` ON DELETE CASCADE

---

## テストケース

マイグレーションはテストコードではなく、実行後の確認で検証する。

### 確認項目

- [ ] `sqlx migrate run` が正常に完了する
- [ ] 各テーブルが作成されている
- [ ] 外部キー制約が正しく動作する（親削除時に子も削除される）
- [ ] UNIQUE 制約が正しく動作する（steps の trial_id + position）

---

## 補足資料

| ファイル | 内容 |
|----------|------|
| [sample_data.sql](./sample_data.sql) | テスト用サンプルデータ |

---

## 完了条件

- [ ] 3 つのマイグレーションファイルが作成されている
- [ ] `sqlx migrate run` が正常に完了する
- [ ] 各テーブルのカラム・制約が設計通りになっている
