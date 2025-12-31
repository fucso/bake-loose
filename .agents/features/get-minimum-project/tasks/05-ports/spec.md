# Task: Ports層（ProjectRepository トレイト）

> Feature: [get-project](../../spec.md)
> 依存: 04-domain

## 目的
ProjectRepository トレイトと RepositoryError を定義する。ドメイン層とリポジトリ層の境界を抽象化する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/ports.rs` | 新規 | ports モジュール（サブモジュールを再公開） |
| `backend/src/ports/error.rs` | 新規 | RepositoryError 定義 |
| `backend/src/ports/project_repository.rs` | 新規 | ProjectRepository トレイト |
| `backend/src/lib.rs` | 修正 | ports モジュールの公開追加 |

---

## 設計詳細

### RepositoryError

リポジトリ操作で発生するエラーの種類を定義:

| バリアント | フィールド | 説明 |
|-----------|-----------|------|
| `NotFound` | `entity: String, id: String` | データが見つからない |
| `Conflict` | `entity: String, field: String` | 一意性制約違反 |
| `Connection` | - | 接続エラー |
| `Internal` | `message: String` | その他の内部エラー |

### ProjectRepository トレイト

今回必要な読み取り操作のみを定義:

- `find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>`
- `find_all(&self) -> Result<Vec<Project>, RepositoryError>`

将来的に save, delete, exists_by_name 等を追加予定。

### Ports層の原則

- ドメイン層にのみ依存
- sqlx などDB固有の型を含まない
- 実装詳細（SQL等）を含まない
- `async_trait` マクロを使用
- `Send + Sync` バウンドを付ける

---

## 完了条件

- [ ] `RepositoryError` が定義されている
- [ ] `ProjectRepository` トレイトが定義されている
- [ ] `find_by_id`, `find_all` メソッドが定義されている
- [ ] ドメイン層にのみ依存している（sqlx への依存なし）
- [ ] `cargo check` が成功する
