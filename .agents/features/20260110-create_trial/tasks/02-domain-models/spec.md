# Task: ドメインモデル定義

> Feature: [create_trial](../../spec.md)
> 依存: なし

## 目的

Trial、Step、Parameter のドメインモデルと、関連する値オブジェクト（Unit, TimeUnit, Duration, ParameterContent など）を定義する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/domain/models.rs` | 修正 | 新規モジュールの公開追加 |
| `src/domain/models/trial.rs` | 新規 | Trial, TrialId, TrialStatus |
| `src/domain/models/step.rs` | 新規 | Step, StepId |
| `src/domain/models/parameter.rs` | 新規 | Parameter, ParameterId, ParameterContent |
| `src/domain/models/unit.rs` | 新規 | Unit（質量、温度、体積、割合） |
| `src/domain/models/time_unit.rs` | 新規 | TimeUnit, Duration |

---

## 設計詳細

### ファイル構成

```
src/domain/models/
├── project.rs      (既存)
├── trial.rs        (新規)
├── step.rs         (新規)
├── parameter.rs    (新規)
├── unit.rs         (新規)
└── time_unit.rs    (新規)
```

### Trial モデル

```rust
// src/domain/models/trial.rs

/// Trial の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrialId(pub Uuid);

/// 試行のステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    InProgress,
    Completed,
}

/// 試行（Trial）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    status: TrialStatus,
    memo: Option<String>,
}
```

**メソッド:**
- `new(project_id: ProjectId) -> Self` - ID 自動生成、ステータスは InProgress
- `from_raw(...)` - DB からの復元用
- ゲッター: `id()`, `project_id()`, `status()`, `memo()`

### Step モデル

```rust
// src/domain/models/step.rs

/// Step の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

/// ステップ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    trial_id: TrialId,
    name: Option<String>,
    position: u32,
    started_at: Option<DateTime<Utc>>,
}
```

**メソッド:**
- `new(trial_id: TrialId, position: u32) -> Self` - ID 自動生成
- `from_raw(...)` - DB からの復元用
- ゲッター: `id()`, `trial_id()`, `name()`, `position()`, `started_at()`

### Parameter モデル

```rust
// src/domain/models/parameter.rs

/// Parameter の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParameterId(pub Uuid);

/// パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    id: ParameterId,
    step_id: StepId,
    content: ParameterContent,
}

/// パラメーターの内容
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterContent {
    KeyValue(KeyValueParameter),
    Text(TextParameter),
    DurationRange(DurationRangeParameter),
    TimePoint(TimePointParameter),
}

/// Key-Value パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueParameter {
    key: String,
    value: ParameterValue,
}

/// パラメーター値
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    Text(String),
    Quantity { amount: i32, unit: Unit },
}

/// テキストパラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextParameter {
    value: String,
}

/// 期間パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurationRangeParameter {
    duration: Duration,
    note: Option<String>,
}

/// 時点パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimePointParameter {
    elapsed: Duration,
    note: String,
}
```

### Unit（単位）

```rust
// src/domain/models/unit.rs

/// 計測単位（時間系を除く）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    // 質量
    Gram,
    Kilogram,

    // 温度
    Celsius,

    // 体積
    Milliliter,
    Liter,

    // 割合
    Percent,
}
```

**メソッド:**
- `as_str(&self) -> &'static str` - DB/JSON 用の文字列表現
- `from_str(s: &str) -> Option<Self>` - 文字列からの変換

### TimeUnit / Duration（時間）

```rust
// src/domain/models/time_unit.rs

use std::time::Duration as StdDuration;

/// 時間の単位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
}

/// 期間（値と表示単位を持つ）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duration {
    value: StdDuration,
    display_unit: TimeUnit,
}
```

**Duration のメソッド:**
- `seconds(value: u64) -> Self`
- `minutes(value: u64) -> Self`
- `hours(value: u64) -> Self`
- `as_std(&self) -> &StdDuration` - 内部値の取得
- `display_unit(&self) -> TimeUnit`
- `display_value(&self) -> u64` - 表示単位での値
- `as_secs(&self) -> u64` - 秒数での取得（比較・計算用）

**TimeUnit のメソッド:**
- `as_str(&self) -> &'static str` - DB/JSON 用
- `from_str(s: &str) -> Option<Self>`

---

## モジュール公開

```rust
// src/domain/models.rs

mod project;
mod trial;
mod step;
mod parameter;
mod unit;
mod time_unit;

pub use project::*;
pub use trial::*;
pub use step::*;
pub use parameter::*;
pub use unit::*;
pub use time_unit::*;
```

---

## テストケース

### テストファイル

- **ユニットテスト**: 各モデルファイル内の `#[cfg(test)] mod tests`

### Trial

| テスト名 | 内容 |
|----------|------|
| `test_trial_new_creates_with_in_progress_status` | new() で InProgress ステータスになる |
| `test_trial_id_generates_unique` | TrialId::new() がユニークな ID を生成 |

### Step

| テスト名 | 内容 |
|----------|------|
| `test_step_new_creates_with_given_position` | new() で指定した position が設定される |

### Parameter

| テスト名 | 内容 |
|----------|------|
| `test_key_value_with_quantity` | KeyValue + Quantity が正しく構築できる |
| `test_key_value_with_text` | KeyValue + Text が正しく構築できる |
| `test_text_parameter` | TextParameter が正しく構築できる |
| `test_duration_range_parameter` | DurationRangeParameter が正しく構築できる |
| `test_time_point_parameter` | TimePointParameter が正しく構築できる |

### Duration

| テスト名 | 内容 |
|----------|------|
| `test_duration_seconds` | seconds() が正しく秒を設定 |
| `test_duration_minutes` | minutes() が正しく分を秒に変換 |
| `test_duration_hours` | hours() が正しく時間を秒に変換 |
| `test_duration_display_value` | display_value() が表示単位での値を返す |
| `test_duration_comparison` | 異なる単位でも秒数での比較が可能 |

### Unit / TimeUnit

| テスト名 | 内容 |
|----------|------|
| `test_unit_as_str_roundtrip` | as_str と from_str が可逆 |
| `test_time_unit_as_str_roundtrip` | as_str と from_str が可逆 |

---

## 完了条件

- [ ] Trial, Step, Parameter モデルが定義されている
- [ ] Unit, TimeUnit, Duration が定義されている
- [ ] 各モデルに new(), from_raw(), ゲッターが実装されている
- [ ] models.rs で全モデルが公開されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
