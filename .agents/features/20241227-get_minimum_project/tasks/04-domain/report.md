# Task Report: Domain層（Project モデル）

> 実施日時: 2025-12-27
> 依存タスク: 01-dependencies

## 実施内容

### 変更ファイル

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/domain/models/project.rs` | 新規 | ProjectId / Project モデル定義 |
| `backend/src/domain/models.rs` | 新規 | models サブモジュール |
| `backend/src/domain.rs` | 新規 | domain モジュール |
| `backend/src/main.rs` | 修正 | `mod domain;` 宣言を追加 |

## ビルド・テスト結果

### cargo check

```
warning: struct `ProjectId` is never constructed
warning: struct `Project` is never constructed
warning: function `create_pool` is never used

Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### cargo test

```
running 2 tests
test domain::models::project::tests::test_project_id_new_generates_unique_ids ... ok
test domain::models::project::tests::test_project_new_creates_with_auto_id ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

## 設計上の議論と決定

### ProjectId の管理レイヤーについて

**議論**:
- `ProjectId` は純粋なビジネスロジックというより「DB管理のための識別子」であり、Domain層に置くべきか Repository層に置くべきか。
- Repository層に置く場合、UseCase層で Domain の `Project`（ID なし）と Repository の ID を合成する必要がある。

**検討した選択肢**:

1. **Domain層に ID を持たせる（採用）**
   - `Project` が `id: ProjectId` を持つ
   - シンプルで実用的

2. **Repository層で ID を管理**
   - Domain の `Project` は ID を持たない
   - UseCase で `ProjectWithId { id: Uuid, project: Project }` のように合成
   - タプル管理が複雑になる

**決定理由**:
- 今回のアプリケーションでは全 Entity が ID を持ち、ID による取得・更新が主要なユースケース
- 合成パターンは実装可能だが、Repository が `(Uuid, Project)` を返す設計が冗長

### 永続化層からの復元方法

**議論**:
- フィールドを `pub` にして直接構築を許可するか
- 専用メソッドを用意するか

**検討した選択肢**:

1. **フィールドを `pub` にする**
   - Repository で `Project { id, name }` と直接構築可能
   - 問題: 外部から自由に変更可能になり、不正な状態を作れてしまう

2. **`from_raw` メソッドを用意（採用）**
   - フィールドは private のまま
   - `Project::from_raw(id, name)` で構築
   - ゲッター経由でのみアクセス可能

**決定理由**:
- フィールドを `pub` にすると不変条件を破壊される可能性がある
- `from_raw` という名前で「生データからの構築」という意図を明示

**命名について**:
- Rust では `from` は `From` トレイト用に予約されており、単一の型変換に使う
- 複数引数の場合は `from_raw` や `from_parts` が適切

## 先送り事項

- [ ] `Project`, `ProjectId` が未使用警告 → 05-ports, 06-repository で使用予定
- [ ] `create_pool` 関数が未使用警告 → 09-integration で使用予定

## 次タスクへの申し送り

### 05-ports（ProjectRepository トレイト）

- `use crate::domain::models::project::{Project, ProjectId};` で参照

### 06-repository（PostgreSQL実装）

- `Project::from_raw(id, name)` を使用して構築
- 例: `Project::from_raw(ProjectId(row.id), row.name)`
- ※ 06-repository/spec.md を修正済み
