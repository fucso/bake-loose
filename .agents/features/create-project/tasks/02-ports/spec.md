# Task: Ports トレイト拡張

> Feature: [create-project](../../spec.md)
> 依存: 01-domain-action

## 目的
`ProjectRepository` トレイトに書き込み系メソッド（`save`, `exists_by_name`）を追加する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/ports/project_repository.rs` | 修正 | `save`, `exists_by_name` メソッドを追加 |

---

## 設計詳細

### 追加メソッド

以下の2つのメソッドを `ProjectRepository` トレイトに追加する。

#### save
プロジェクトを保存（新規作成または更新）する。

- **シグネチャ**: `async fn save(&self, project: &Project) -> Result<(), RepositoryError>`
- **動作**:
  - 存在しない場合は INSERT
  - 存在する場合は UPDATE（UPSERT パターン）
- **エラー**:
  - `RepositoryError::Connection` - 接続エラー
  - `RepositoryError::Internal` - その他の内部エラー

#### exists_by_name
指定した名前のプロジェクトが存在するかを確認する。

- **シグネチャ**: `async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>`
- **用途**: 重複チェック（ユースケース層で使用）
- **エラー**:
  - `RepositoryError::Connection` - 接続エラー
  - `RepositoryError::Internal` - その他の内部エラー

### メソッド順序

既存メソッドとの一貫性を考慮し、以下の順序で配置する:

1. `find_by_id` (既存)
2. `find_all` (既存)
3. `exists_by_name` (新規)
4. `save` (新規)

---

## テストケース

Ports 層はトレイト定義のみのため、単体テストは不要。
テストは Repository 層（03-repository）および UseCase 層（04-use-case）で実施する。

---

## 完了条件

- [ ] `save` メソッドがトレイトに定義されている
- [ ] `exists_by_name` メソッドがトレイトに定義されている
- [ ] ドメイン層の型のみを使用している（外部クレートへの依存なし）
- [ ] `cargo check` が通る
