# Task: PgTrialRepository 実装

> Feature: [Trial モデルと関連アクション](../../spec.md)
> 依存: 01-migration, 04-ports

## 目的

TrialRepository トレイトの PostgreSQL 実装を提供する。Trial を aggregate root として、Steps と Parameters を含めて操作する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `backend/src/repository/trial_repo.rs` | 新規 | PgTrialRepository |
| `backend/src/repository/models/trial_row.rs` | 新規 | TrialRow, StepRow, ParameterRow |
| `backend/src/repository/models.rs` | 修正 | モジュール追加 |
| `backend/src/repository/pg_unit_of_work.rs` | 修正 | trial_repository 追加 |
| `backend/src/repository.rs` | 修正 | モジュール追加 |

---

## 設計詳細

### DB モデル（Row 構造体）

#### TrialRow

```rust
#[derive(Debug, FromRow)]
pub struct TrialRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### StepRow

```rust
#[derive(Debug, FromRow)]
pub struct StepRow {
    pub id: Uuid,
    pub trial_id: Uuid,
    pub name: String,
    pub position: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### ParameterRow

```rust
#[derive(Debug, FromRow)]
pub struct ParameterRow {
    pub id: Uuid,
    pub step_id: Uuid,
    pub content: serde_json::Value,  // JSONB
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### SortColumn 実装

```rust
impl SortColumn for TrialSortColumn {
    fn as_sql_column(&self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
```

### PgTrialRepository 実装

#### find_by_id

1. trials テーブルから Trial を取得
2. steps テーブルから関連する Steps を取得（position 順）
3. parameters テーブルから関連する Parameters を取得
4. 階層構造に組み立てて Trial を返す

#### find_by_project_id

1. trials テーブルからプロジェクトに紐づく Trial 一覧を取得
2. 各 Trial について Steps と Parameters を取得
3. 階層構造に組み立てて返す

#### save

1. Trial を UPSERT（ON CONFLICT DO UPDATE）
2. 既存の Steps と Parameters を削除
3. 新しい Steps と Parameters を INSERT

#### delete

- trials テーブルから削除（CASCADE で Steps、Parameters も削除）

### 注意点

- ParameterContent は JSONB として保存
- JSONB からドメインモデルへの変換は serde を使用
- save は全置換方式（既存データを削除して新規挿入）
- N+1 問題を避けるため、find_by_project_id では IN 句で一括取得を検討

---

## テストケース

### テストファイル

- **統合テスト**: `backend/src/repository/trial_repo.rs` 内の `#[cfg(test)] mod tests`（`#[sqlx::test]` 使用）

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_save_and_find_trial` | Trial を保存して取得できる |
| `test_save_trial_with_steps` | Steps を含む Trial を保存して取得できる |
| `test_save_trial_with_parameters` | Parameters を含む Trial を保存して取得できる |
| `test_find_by_project_id` | プロジェクト別に Trial を取得できる |
| `test_update_trial` | 既存の Trial を更新できる |
| `test_delete_trial` | Trial を削除できる |
| `test_delete_cascades_to_steps` | Trial 削除時に Steps も削除される |
| `test_parameter_content_json_roundtrip` | ParameterContent の JSON 変換が正しく動作する |

---

## 完了条件

- [ ] TrialRow, StepRow, ParameterRow が定義されている
- [ ] PgTrialRepository が TrialRepository トレイトを実装している
- [ ] ParameterContent の JSONB 変換が正しく動作する
- [ ] PgUnitOfWork に trial_repository が追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
