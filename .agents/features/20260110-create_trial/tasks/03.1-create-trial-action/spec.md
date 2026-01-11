# Task: Trial 作成アクション

> Feature: [create_trial](../../spec.md)
> 依存: [02-domain-models](../02-domain-models/)

## 目的

Trial のみを作成するドメインアクション（create_trial）を実装する。validate / execute / run パターンに従う。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/domain/actions.rs` | 修正 | trial モジュールの追加 |
| `src/domain/actions/trial.rs` | 新規 | trial アクションモジュール |
| `src/domain/actions/trial/create_trial.rs` | 新規 | create_trial アクション |

---

## 設計詳細

### ファイル構成

```
src/domain/actions/
├── project.rs      (既存)
├── project/        (既存)
├── trial.rs        (新規)
└── trial/
    └── create_trial.rs  (新規)
```

### create_trial アクション

Trial のみを作成する。Step や Parameter は含まない。

**Command 構造:**

```rust
// src/domain/actions/trial/create_trial.rs

pub struct Command {
    pub project_id: ProjectId,
    pub memo: Option<String>,
}
```

**エラー型:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Project との紐付けがない（project_id が空の UUID）
    EmptyProjectId,
}
```

**validate 関数:**

```rust
pub fn validate(command: &Command) -> Result<(), Error> {
    // project_id が空（nil UUID）でないことを確認
    if command.project_id.0.is_nil() {
        return Err(Error::EmptyProjectId);
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
    let trial = Trial::new(command.project_id, command.memo);
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

- **ユニットテスト**: `src/domain/actions/trial/create_trial.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_create_trial_with_memo` | memo 付きで Trial を作成できる |
| `test_create_trial_without_memo` | memo なしで Trial を作成できる |
| `test_trial_status_is_in_progress` | 作成直後の Trial は InProgress |
| `test_trial_has_correct_project_id` | 指定した project_id が設定される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_returns_error_when_project_id_is_nil` | project_id が nil UUID の場合 EmptyProjectId エラー |

---

## 完了条件

- [ ] create_trial アクションが validate / execute / run パターンで実装されている
- [ ] Command が定義されている
- [ ] Error 型に EmptyProjectId が定義されている
- [ ] project_id が nil UUID の場合はエラーになる
- [ ] Output が Trial のみを含む
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
