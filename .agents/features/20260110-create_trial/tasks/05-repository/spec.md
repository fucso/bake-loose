# Task: Repository 実装

> Feature: [create_trial](../../spec.md)
> 依存: [01-migration](../01-migration/), [04-ports](../04-ports/)

## 目的

TrialRepository トレイトの PostgreSQL 実装（PgTrialRepository）と、DB モデル（Row 構造体）を作成する。

---

## 変更対象

| ファイル | 操作 | 概要 |
|----------|------|------|
| `src/repository.rs` | 修正 | 新規モジュールの追加 |
| `src/repository/models.rs` | 修正 | 新規 Row モジュールの追加 |
| `src/repository/models/trial_row.rs` | 新規 | TrialRow |
| `src/repository/models/step_row.rs` | 新規 | StepRow |
| `src/repository/models/parameter_row.rs` | 新規 | ParameterRow, JSONB 変換 |
| `src/repository/trial_repo.rs` | 新規 | PgTrialRepository |
| `src/repository/pg_unit_of_work.rs` | 修正 | trial_repository() の追加 |

---

## 設計詳細

### TrialRow

```rust
// src/repository/models/trial_row.rs

#[derive(Debug, FromRow)]
pub struct TrialRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TrialRow> for Trial {
    fn from(row: TrialRow) -> Self {
        Trial::from_raw(
            TrialId(row.id),
            ProjectId(row.project_id),
            TrialStatus::from_str(&row.status).unwrap_or(TrialStatus::InProgress),
            row.memo,
        )
    }
}
```

### StepRow

```rust
// src/repository/models/step_row.rs

#[derive(Debug, FromRow)]
pub struct StepRow {
    pub id: Uuid,
    pub trial_id: Uuid,
    pub name: Option<String>,
    pub position: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<StepRow> for Step {
    fn from(row: StepRow) -> Self {
        Step::from_raw(
            StepId(row.id),
            TrialId(row.trial_id),
            row.name,
            row.position as u32,
            row.started_at,
        )
    }
}
```

### ParameterRow と JSONB 変換

```rust
// src/repository/models/parameter_row.rs

#[derive(Debug, FromRow)]
pub struct ParameterRow {
    pub id: Uuid,
    pub step_id: Uuid,
    pub content_type: String,
    pub content: serde_json::Value,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**JSONB → ParameterContent 変換:**

```rust
impl TryFrom<ParameterRow> for Parameter {
    type Error = ParameterConversionError;

    fn try_from(row: ParameterRow) -> Result<Self, Self::Error> {
        let content = match row.content_type.as_str() {
            "key_value" => parse_key_value(&row.content)?,
            "text" => parse_text(&row.content)?,
            "duration_range" => parse_duration_range(&row.content)?,
            "time_point" => parse_time_point(&row.content)?,
            _ => return Err(ParameterConversionError::UnknownContentType),
        };

        Ok(Parameter::from_raw(
            ParameterId(row.id),
            StepId(row.step_id),
            content,
        ))
    }
}
```

**ParameterContent → JSONB 変換:**

```rust
pub fn parameter_to_json(param: &Parameter) -> (String, serde_json::Value) {
    match param.content() {
        ParameterContent::KeyValue(kv) => {
            let value_json = match &kv.value {
                ParameterValue::Text(t) => json!({
                    "type": "text",
                    "text": t
                }),
                ParameterValue::Quantity { amount, unit } => json!({
                    "type": "quantity",
                    "amount": amount,
                    "unit": unit.as_str()
                }),
            };
            ("key_value".to_string(), json!({
                "key": kv.key,
                "value": value_json
            }))
        }
        ParameterContent::Text(t) => {
            ("text".to_string(), json!({ "value": t.value }))
        }
        ParameterContent::DurationRange(dr) => {
            ("duration_range".to_string(), json!({
                "duration_seconds": dr.duration.as_secs(),
                "display_unit": dr.duration.display_unit().as_str(),
                "note": dr.note
            }))
        }
        ParameterContent::TimePoint(tp) => {
            ("time_point".to_string(), json!({
                "elapsed_seconds": tp.elapsed.as_secs(),
                "display_unit": tp.elapsed.display_unit().as_str(),
                "note": tp.note
            }))
        }
    }
}
```

### PgTrialRepository

```rust
// src/repository/trial_repo.rs

pub struct PgTrialRepository {
    executor: PgExecutor,
}

#[async_trait]
impl TrialRepository for PgTrialRepository {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError> {
        let query = sqlx::query_as::<_, TrialRow>(
            "SELECT * FROM trials WHERE id = $1"
        ).bind(id.0);

        self.executor
            .fetch_optional(query)
            .await
            .map(|row| row.map(Trial::from))
            .map_err(|e| RepositoryError::Internal { message: e.to_string() })
    }

    async fn save_all(
        &self,
        trial: &Trial,
        steps: &[Step],
        parameters: &[Parameter],
    ) -> Result<(), RepositoryError> {
        // Trial を保存
        self.save_trial(trial).await?;

        // Steps を保存
        for step in steps {
            self.save_step(step).await?;
        }

        // Parameters を保存
        for param in parameters {
            self.save_parameter(param).await?;
        }

        Ok(())
    }

    // ... 他のメソッド
}
```

### PgUnitOfWork への追加

```rust
// src/repository/pg_unit_of_work.rs

impl UnitOfWork for PgUnitOfWork {
    type ProjectRepo = PgProjectRepository;
    type TrialRepo = PgTrialRepository;  // 追加

    fn trial_repository(&mut self) -> Self::TrialRepo {
        PgTrialRepository::new(self.executor())
    }

    // ...
}
```

---

## テストケース

### テストファイル

- **統合テスト**: `src/repository/trial_repo.rs` 内の `#[cfg(test)] mod tests`
- **フィクスチャ**: `tests/fixtures/trials.sql`

### 正常系

| テスト名 | 内容 |
|----------|------|
| `test_save_and_find_trial` | Trial を保存して取得できる |
| `test_find_by_project_id` | Project ID で Trial を取得できる |
| `test_save_and_find_steps` | Steps を保存して取得できる |
| `test_steps_ordered_by_position` | Steps が position 順で取得される |
| `test_save_and_find_parameters` | Parameters を保存して取得できる |
| `test_save_all_in_transaction` | save_all で一括保存できる |
| `test_cascade_delete_on_trial_delete` | Trial 削除時に Steps, Parameters も削除される |

### Parameter 種別ごとのテスト

| テスト名 | 内容 |
|----------|------|
| `test_key_value_text_roundtrip` | KeyValue (Text) の保存・取得 |
| `test_key_value_quantity_roundtrip` | KeyValue (Quantity) の保存・取得 |
| `test_text_parameter_roundtrip` | TextParameter の保存・取得 |
| `test_duration_range_roundtrip` | DurationRangeParameter の保存・取得 |
| `test_time_point_roundtrip` | TimePointParameter の保存・取得 |

---

## 完了条件

- [ ] TrialRow, StepRow, ParameterRow が定義されている
- [ ] JSONB ↔ ParameterContent の変換が実装されている
- [ ] PgTrialRepository が TrialRepository トレイトを実装している
- [ ] PgUnitOfWork に trial_repository() が追加されている
- [ ] 上記テストケースがすべて実装されている
- [ ] テストが通る
