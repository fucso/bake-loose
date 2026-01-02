# Task: UseCase 実装

> Feature: [create-project](../../spec.md)
> 依存: 03-repository

## 目的
`create_project` ユースケースを実装し、ドメインアクションと永続化を組み合わせる。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/use_case/project/create_project.rs` | 新規 | create_project ユースケース |
| `src/use_case/project.rs` | 修正 | create_project モジュールを追加 |

---

## 設計詳細

### ユースケースの流れ

1. **重複チェック** (DB検証を先に行う)
   - `exists_by_name` で同名プロジェクトの存在を確認
   - 存在する場合は `Error::DuplicateName` を返す

2. **ドメインアクション実行**
   - `domain::actions::project::create_project::run()` を呼び出す
   - バリデーションエラーは `Error::Domain(...)` でラップ

3. **永続化**
   - `project_repository().save()` で DB に保存

4. **コミット**
   - `uow.commit()` でトランザクションを確定

5. **結果返却**
   - 作成された `Project` を返す

### エラー型

```rust
pub enum Error {
    Domain(create_project::Error),  // ドメインアクションのエラー
    DuplicateName,                   // 同名プロジェクト存在
    Infrastructure(String),          // DB エラー等
}
```

### Input 型

```rust
pub struct Input {
    pub name: String,
}
```

シンプルに `name` のみを受け取る。

### execute 関数シグネチャ

```rust
pub async fn execute<U: UnitOfWork>(
    uow: &mut U,
    input: Input,
) -> Result<Project, Error>
```

---

## テストケース

ファイル: `src/use_case/project/create_project.rs` 内の `#[cfg(test)] mod tests`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_execute_creates_project_successfully` | 有効な入力でプロジェクトが作成される |

### 異常系

| テスト名 | 内容 |
|----------|------|
| `test_execute_returns_duplicate_error_when_name_exists` | 同名プロジェクトが存在する場合 `DuplicateName` エラー |
| `test_execute_returns_domain_error_for_empty_name` | 空の名前で `Domain(EmptyName)` エラー |
| `test_execute_returns_domain_error_for_too_long_name` | 長すぎる名前で `Domain(NameTooLong)` エラー |

### テスト実装のポイント

- `MockUnitOfWork` を使用してリポジトリをモック化
- 重複チェックのテストでは、モックに既存プロジェクトを設定

---

## 完了条件

- [ ] 重複チェックがドメインアクション実行前に行われている
- [ ] ドメインアクションのエラーが適切にラップされている
- [ ] 永続化後に `commit()` が呼ばれている
- [ ] 上記テストケースがすべて実装されている
- [ ] `cargo test` が通る
