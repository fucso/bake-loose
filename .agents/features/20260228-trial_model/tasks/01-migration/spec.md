# Task: マイグレーション

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: なし

## 目的

Trial、Step、Parameter を格納するためのデータベーステーブルを作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/migrations/{timestamp}_create_trials.sql` | 新規 | trials, steps, parameters テーブル作成 |

---

## 設計詳細

### テーブル設計

#### trials テーブル

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | 識別子 |
| project_id | UUID | NOT NULL, FK | 所属プロジェクト |
| name | VARCHAR(100) | NULL | 任意の名前 |
| memo | TEXT | NULL | 備考・ノート |
| status | VARCHAR(20) | NOT NULL, DEFAULT 'in_progress' | ステータス |
| created_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 更新日時 |

- `project_id` に対して外部キー制約を設定
- `project_id` に対してインデックスを作成（プロジェクト別の Trial 一覧取得用）

#### steps テーブル

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | 識別子 |
| trial_id | UUID | NOT NULL, FK | 所属 Trial |
| name | VARCHAR(100) | NOT NULL | ステップ名 |
| position | INTEGER | NOT NULL | 順序（0始まり） |
| started_at | TIMESTAMP WITH TIME ZONE | NULL | 開始日時 |
| completed_at | TIMESTAMP WITH TIME ZONE | NULL | 完了日時 |
| created_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 更新日時 |

- `trial_id` に対して外部キー制約（CASCADE DELETE）を設定
- `(trial_id, position)` に対してユニーク制約を設定

#### parameters テーブル

| カラム | 型 | 制約 | 説明 |
|--------|-----|------|------|
| id | UUID | PRIMARY KEY | 識別子 |
| step_id | UUID | NOT NULL, FK | 所属 Step |
| content | JSONB | NOT NULL | パラメーター内容 |
| created_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | 更新日時 |

- `step_id` に対して外部キー制約（CASCADE DELETE）を設定
- `content` カラムは ParameterContent の JSON 表現を格納

### JSONB の content 構造

```json
// KeyValue
{
  "type": "key_value",
  "key": "強力粉",
  "value": { "type": "quantity", "amount": 300, "unit": "g" }
}

// Duration
{
  "type": "duration",
  "duration": { "value": 12, "unit": "h" },
  "note": "冷蔵発酵"
}

// TimeMarker
{
  "type": "time_marker",
  "at": { "value": 5, "unit": "min" },
  "note": "スチーム噴射"
}

// Text
{
  "type": "text",
  "value": "生地を3分割し..."
}
```

---

## 完了条件

- [ ] trials テーブルが作成されている
- [ ] steps テーブルが作成されている
- [ ] parameters テーブルが作成されている
- [ ] 外部キー制約が正しく設定されている
- [ ] CASCADE DELETE が設定されている（Trial 削除時に Step/Parameter も削除）
- [ ] マイグレーションが正常に実行できる
