# Task: Domain層（Project モデル）

> Feature: [get-project](../../spec.md)
> 依存: 01-dependencies

## 目的
Project ドメインモデルを定義する。最小限のフィールド（ID, name）で開始し、将来的に拡張可能な設計とする。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain.rs` | 新規 | domain モジュール（models を再公開） |
| `backend/src/domain/models.rs` | 新規 | models サブモジュール（project を再公開） |
| `backend/src/domain/models/project.rs` | 新規 | Project モデル定義 |
| `backend/src/lib.rs` | 修正 | domain モジュールの公開追加 |

---

## 設計詳細

### ProjectId（NewType パターン）

型安全性のため、UUID をラップした NewType を定義する。

- `ProjectId(pub Uuid)`
- `new()`: 新しいUUIDを生成
- derive: `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`

### Project モデル

最小限のフィールドで開始:

| フィールド | 型 | 説明 |
|------------|-----|------|
| `id` | `ProjectId` | プロジェクトID |
| `name` | `String` | プロジェクト名 |

### メソッド

モデルにはファクトリメソッドとゲッターのみを定義（ビジネスロジックはアクションに委譲）:

- `new(name: String) -> Self`: 新規プロジェクト作成
- `reconstruct(id: ProjectId, name: String) -> Self`: 永続化からの復元用
- `id(&self) -> &ProjectId`: ID のゲッター
- `name(&self) -> &str`: 名前のゲッター

### Domain層の原則

- 外部クレートへの依存は `uuid`, `serde` のみ
- I/O 操作は行わない
- 永続化の詳細を知らない

---

## 完了条件

- [ ] `ProjectId` が NewType パターンで定義されている
- [ ] `Project` モデルが定義されている
- [ ] ゲッターが実装されている
- [ ] `reconstruct` メソッドが実装されている（リポジトリからの復元用）
- [ ] `cargo check` が成功する
