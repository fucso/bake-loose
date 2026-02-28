# Task: ドメインモデル

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: なし

## 目的

Trial、Step、Parameter のドメインモデルと関連する値オブジェクトを定義する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/models/trial.rs` | 新規 | Trial, TrialId, TrialStatus |
| `backend/src/domain/models/step.rs` | 新規 | Step, StepId |
| `backend/src/domain/models/parameter.rs` | 新規 | Parameter, ParameterId, ParameterContent, ParameterValue, DurationValue |
| `backend/src/domain/models.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### Trial モデル

- `TrialId`: NewType パターンで UUID をラップ
- `TrialStatus`: enum で InProgress / Completed を定義
- `Trial`:
  - フィールド: id, project_id, name, memo, status, steps, created_at, updated_at
  - steps は `Vec<Step>` として保持（aggregate root として子を含む）
  - ファクトリメソッド `new()` と再構築用 `from_raw()` を提供
  - ゲッターのみ、ロジックはアクションに集約

### Step モデル

- `StepId`: NewType パターンで UUID をラップ
- `Step`:
  - フィールド: id, trial_id, name, position, started_at, completed_at, parameters, created_at, updated_at
  - parameters は `Vec<Parameter>` として保持
  - ファクトリメソッド `new()` と再構築用 `from_raw()` を提供

### Parameter モデル

- `ParameterId`: NewType パターンで UUID をラップ
- `Parameter`:
  - フィールド: id, step_id, content, created_at, updated_at
  - ファクトリメソッド `new()` と再構築用 `from_raw()` を提供

### ParameterContent (enum)

```
KeyValue { key: String, value: ParameterValue }
Duration { duration: DurationValue, note: Option<String> }
TimeMarker { at: DurationValue, note: String }
Text { value: String }
```

### ParameterValue (enum)

```
Text(String)
Quantity { amount: f64, unit: String }
```

### DurationValue (struct)

```
DurationValue { value: f64, unit: String }
```

### 注意点

- すべてのモデルは `serde::Serialize, Deserialize` を derive
- 外部クレート依存は serde のみ（domain 層の原則）
- created_at / updated_at は chrono::DateTime<Utc> を使用
- Trial は aggregate root として Steps を含む（リポジトリは Trial 単位で保存・取得）

---

## テストケース

### テストファイル

- **ユニットテスト**: 各モデルファイル内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_trial_id_new_generates_unique_ids` | TrialId::new() が一意の ID を生成する |
| `test_trial_new_creates_with_in_progress_status` | Trial::new() が InProgress ステータスで作成される |
| `test_step_id_new_generates_unique_ids` | StepId::new() が一意の ID を生成する |
| `test_parameter_content_key_value_with_quantity` | KeyValue に Quantity を設定できる |
| `test_parameter_content_duration_with_note` | Duration に note を設定できる |
| `test_duration_value_creation` | DurationValue が正しく作成される |

---

## 完了条件

- [ ] Trial, TrialId, TrialStatus が定義されている
- [ ] Step, StepId が定義されている
- [ ] Parameter, ParameterId, ParameterContent, ParameterValue, DurationValue が定義されている
- [ ] すべてのモデルに Serialize, Deserialize が derive されている
- [ ] domain/models.rs にモジュールが追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
