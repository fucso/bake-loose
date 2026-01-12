# Task: UseCase層（get_project, list_projects）

> Feature: [get-project](../../spec.md)
> 依存: 05-ports

## 目的
プロジェクト取得のユースケースを実装する。今回は読み取り専用のため、シンプルなリポジトリ経由の取得のみ。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/use_case.rs` | 新規 | use_case モジュール（サブモジュールを再公開） |
| `backend/src/use_case/project.rs` | 新規 | project サブモジュール（ユースケースを再公開） |
| `backend/src/use_case/project/get_project.rs` | 新規 | 単一プロジェクト取得 |
| `backend/src/use_case/project/list_projects.rs` | 新規 | プロジェクト一覧取得 |
| `backend/src/lib.rs` | 修正 | use_case モジュールの公開追加 |

---

## 設計詳細

### get_project ユースケース

IDでプロジェクトを取得する:

- 入力: `ProjectId`
- 出力: `Result<Option<Project>, Error>`
- エラー: `Error::Infrastructure(String)` のみ（読み取り専用のため）

### list_projects ユースケース

すべてのプロジェクトを取得する:

- 入力: なし
- 出力: `Result<Vec<Project>, Error>`
- エラー: `Error::Infrastructure(String)` のみ

### UseCase層のシンプル化

今回は読み取り専用のため:
- ドメインアクションの呼び出しなし
- DB問い合わせが必要な検証なし
- トランザクション管理不要

単純にリポジトリを呼び出してエラーを変換するだけの薄い層となる。

### 将来の拡張

create_project などの書き込み系ユースケースを追加する際は:
- UnitOfWork パターンの導入
- ドメインアクションの呼び出し
- DB問い合わせが必要な検証（名前重複チェック等）

---

## 完了条件

- [ ] `get_project` ユースケースが実装されている
- [ ] `list_projects` ユースケースが実装されている
- [ ] ports 層のトレイトのみに依存している（repository 層への直接依存なし）
- [ ] `cargo check` が成功する
