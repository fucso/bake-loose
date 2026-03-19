# Task: Trial 完了アクション

> Feature: [create_trial](../../spec.md)
> 依存: [02-domain-models](../02-domain-models/), [03.1-create-trial-action](../03.1-create-trial-action/)

## 目的

Trial のステータスを Completed に変更するドメインアクション（complete_trial）を実装する。validate / execute / run パターンに従う。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/domain/models/trial.rs` | 修正 | complete メソッドの追加 |
| `src/domain/actions/trial.rs` | 修正 | complete_trial モジュールの追加 |
| `src/domain/actions/trial/complete_trial.rs` | 新規 | complete_trial アクション |

---

## 設計詳細

### ファイル構成

```
src/domain/actions/
├── project.rs      (既存)
├── project/        (既存)
├── trial.rs        (既存 - 修正)
└── trial/
    ├── create_trial.rs    (既存)
    ├── create_step.rs     (既存)
    └── complete_trial.rs  (新規)
```

### Trial モデルへの追加

```rust
// src/domain/models/trial.rs に追加

impl Trial {
    /// Trial を完了状態にする
    pub fn complete(&mut self) {
        self.status = TrialStatus::Completed;
    }
}
```

### complete_trial アクション

Trial のステータスを Completed に変更する。

**Command 構造:**

```rust
// src/domain/actions/trial/complete_trial.rs

pub struct Command {
    pub trial: Trial,
}
```

**エラー型:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 既に完了済み
    AlreadyCompleted,
}
```

**validate 関数:**

```rust
pub fn validate(command: &Command) -> Result<(), Error> {
    if command.trial.status() == TrialStatus::Completed {
        return Err(Error::AlreadyCompleted);
    }
    Ok(())
}
```

**execute 関数:**

```rust
pub struct Output {
    pub trial: Trial,
}

pub fn execute(command: Command) -> Output {
    let mut trial = command.trial;
    trial.complete();
    Output { trial }
}
```

**run 関数:**

```rust
pub fn run(command: Command) -> Result<Output, Error> {
    validate(&command)?;
    Ok(execute(command))
}
```

---

## テストケース

### テストファイル

- **ユニットテスト**: `src/domain/actions/trial/complete_trial.rs` 内の `#[cfg(test)] mod tests`
- **Trial モデルテスト**: `src/domain/models/trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_complete_trial_changes_status` | InProgress から Completed に変更できる |
| `test_completed_trial_has_correct_id` | 完了後も trial_id は変わらない |
| `test_completed_trial_has_correct_project_id` | 完了後も project_id は変わらない |
| `test_completed_trial_has_correct_memo` | 完了後も memo は変わらない |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_already_completed` | 既に Completed の場合 AlreadyCompleted エラー |

### Trial モデルテスト

| テスト名 | 内容 |
|----------|------|
| `test_trial_complete_changes_status` | complete() でステータスが Completed になる |

---

## 完了条件

- [ ] Trial モデルに complete() メソッドが実装されている
- [ ] complete_trial アクションが validate / execute / run パターンで実装されている
- [ ] Command が定義されている
- [ ] Error 型に AlreadyCompleted が定義されている
- [ ] 既に完了済みの Trial は再度完了できない
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
