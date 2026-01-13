# Task: 集約モデルへのリファクタリング

> Feature: [create_trial](../../spec.md)
> 依存: [02-domain-models](../02-domain-models/)

## 目的

Trial を集約ルート（Aggregate Root）として、Trial が Steps を、Step が Parameters を保持する構造にリファクタリングする。
また、Step の `started_at` を必須フィールドに変更する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/models/trial.rs` | 修正 | Trial に steps フィールドを追加 |
| `backend/src/domain/models/trial/step.rs` | 修正 | Step に parameters フィールドを追加、trial_id を削除、started_at を必須に |
| `backend/src/domain/models/trial/parameter.rs` | 修正 | Parameter から step_id を削除 |
| `backend/migrations/{timestamp}_alter_steps_started_at_not_null.sql` | 新規 | started_at を NOT NULL に変更 |

---

## 設計詳細

### 変更前の構造

```
Trial { id, project_id, status, memo }
Step { id, trial_id, name, position, started_at: Option }
Parameter { id, step_id, content }
```

### 変更後の構造

```
Trial {
    id, project_id, status, memo,
    steps: Vec<Step>
}

Step {
    id, name, position, started_at: DateTime<Utc>,  // 必須に変更
    parameters: Vec<Parameter>
}

Parameter {
    id, content
}
```

### マイグレーション

```sql
-- backend/migrations/{timestamp}_alter_steps_started_at_not_null.sql

-- started_at を NOT NULL に変更
-- 既存データがある場合は created_at の値で埋める
UPDATE steps SET started_at = created_at WHERE started_at IS NULL;

ALTER TABLE steps ALTER COLUMN started_at SET NOT NULL;
```

### Trial モデル

```rust
// backend/src/domain/models/trial.rs

/// 試行（Trial）- 集約ルート
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    status: TrialStatus,
    memo: Option<String>,
    steps: Vec<Step>,
}

impl Trial {
    /// 新しい Trial を作成する（Steps は空）
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            id: TrialId::new(),
            project_id,
            status: TrialStatus::InProgress,
            memo: None,
            steps: Vec::new(),
        }
    }

    /// DB などから Trial を復元する
    pub fn from_raw(
        id: TrialId,
        project_id: ProjectId,
        status: TrialStatus,
        memo: Option<String>,
        steps: Vec<Step>,
    ) -> Self {
        Self {
            id,
            project_id,
            status,
            memo,
            steps,
        }
    }

    // Getters
    pub fn id(&self) -> &TrialId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn status(&self) -> TrialStatus { self.status }
    pub fn memo(&self) -> Option<&str> { self.memo.as_deref() }
    pub fn steps(&self) -> &[Step] { &self.steps }

    /// 次の Step の position を取得
    pub fn next_step_position(&self) -> u8 {
        self.steps
            .iter()
            .map(|s| s.position())
            .max()
            .map(|max| max + 1)
            .unwrap_or(0)
    }

    /// Step を追加する
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }
}
```

### Step モデル

```rust
// backend/src/domain/models/trial/step.rs

/// ステップ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    name: Option<String>,
    position: u8,
    started_at: DateTime<Utc>,  // 必須
    parameters: Vec<Parameter>,
}

impl Step {
    /// 新しい Step を作成する（Parameters は空）
    pub fn new(position: u8, started_at: DateTime<Utc>) -> Self {
        Self {
            id: StepId::new(),
            name: None,
            position,
            started_at,
            parameters: Vec::new(),
        }
    }

    /// DB などから Step を復元する
    pub fn from_raw(
        id: StepId,
        name: Option<String>,
        position: u8,
        started_at: DateTime<Utc>,
        parameters: Vec<Parameter>,
    ) -> Self {
        Self {
            id,
            name,
            position,
            started_at,
            parameters,
        }
    }

    // Getters
    pub fn id(&self) -> &StepId { &self.id }
    pub fn name(&self) -> Option<&str> { self.name.as_deref() }
    pub fn position(&self) -> u8 { self.position }
    pub fn started_at(&self) -> DateTime<Utc> { self.started_at }  // Option ではなく直接返す
    pub fn parameters(&self) -> &[Parameter] { &self.parameters }

    /// Parameter を追加する
    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(parameter);
    }
}
```

### Parameter モデル

```rust
// backend/src/domain/models/trial/parameter.rs

/// パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    id: ParameterId,
    content: ParameterContent,
}

impl Parameter {
    pub fn new(content: ParameterContent) -> Self {
        Self {
            id: ParameterId::new(),
            content,
        }
    }

    pub fn from_raw(id: ParameterId, content: ParameterContent) -> Self {
        Self { id, content }
    }

    pub fn id(&self) -> &ParameterId { &self.id }
    pub fn content(&self) -> &ParameterContent { &self.content }
}
```

---

## 備考

### DB スキーマとの対応

steps テーブルには `trial_id`、parameters テーブルには `step_id` が残ります。
これはリポジトリ層で適切にマッピングします：

- Trial を保存する際: Steps から trial_id を設定、Parameters から step_id を設定
- Trial を取得する際: trial_id と step_id を使って関連を解決し、集約として組み立てる

### started_at の必須化

Step は開始時刻が必須になります。これは「いつ開始したか」が Step の重要な属性であるためです。
既存データは `created_at` の値で埋めることで、マイグレーションを安全に実行できます。

---

## テストケース

### テストファイル

- **ユニットテスト**: 各モデルファイル内の `#[cfg(test)] mod tests`

### Trial

| テスト名 | 内容 |
|----------|------|
| `test_trial_new_has_empty_steps` | new() で steps が空 |
| `test_trial_next_step_position_empty` | steps が空の場合 0 を返す |
| `test_trial_next_step_position_with_steps` | steps がある場合 max + 1 を返す |
| `test_trial_add_step` | add_step で Step が追加される |

### Step

| テスト名 | 内容 |
|----------|------|
| `test_step_new_has_empty_parameters` | new() で parameters が空 |
| `test_step_new_has_started_at` | new() で started_at が設定される |
| `test_step_add_parameter` | add_parameter で Parameter が追加される |

### Parameter

| テスト名 | 内容 |
|----------|------|
| `test_parameter_new` | new() で Parameter が作成される |
| `test_parameter_from_raw` | from_raw() で Parameter が復元される |

---

## 完了条件

- [ ] マイグレーションで started_at が NOT NULL に変更されている
- [ ] Trial が `steps: Vec<Step>` を持つ
- [ ] Step が `parameters: Vec<Parameter>` を持つ
- [ ] Step の `started_at` が `DateTime<Utc>` 型（必須）になっている
- [ ] Step から `trial_id` が削除されている
- [ ] Parameter から `step_id` が削除されている
- [ ] Trial に `next_step_position()` メソッドがある
- [ ] Trial に `add_step()` メソッドがある
- [ ] Step に `add_parameter()` メソッドがある
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
