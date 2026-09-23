---
paths: backend/src/repository/**/*.rs
---

# Repository Layer

リポジトリ層はports層で定義されたトレイトの具体実装を担当。PostgreSQL/SQLxを使用。

## 基本原則

- **実装対象**: ports層のトレイト
- **依存先**: ports層、domain層、infrastructure層
- **禁止**: ビジネスロジック、トランザクション管理

**やること**:
- portsトレイトの実装
- SQL発行
- ドメインモデル ↔ DBモデル変換

**やらないこと**:
- ビジネスロジック
- バリデーション
- トランザクション境界の管理

## ファイル配置

```
backend/src/repository/
├── executor.rs         # PgExecutor（Pool/Transaction の抽象化）
├── pg_unit_of_work.rs  # PgUnitOfWork 実装
├── project_repo.rs
├── trial_repo.rs
├── ...
└── models/
    ├── project_row.rs
    ├── trial_row.rs
    └── ...
```

## DBモデル（Row構造体）

ドメインモデルとは別に定義し、変換処理を経由:

```rust
// src/repository/models/project_row.rs

#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    // ...
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Project::reconstruct(ProjectId(row.id), row.name, ...)
    }
}
```

## SortColumn の実装

ports層で定義された `{Model}SortColumn` に対して、DBカラム名へのマッピングを実装する:

```rust
// src/repository/models/project_row.rs

use crate::ports::project_repository::ProjectSortColumn;
use crate::ports::sort::SortColumn;

impl SortColumn for ProjectSortColumn {
    fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
```

リポジトリでは `Sort<C>` の `to_order_by_clause()` を使用してSQLを生成:

```rust
async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError> {
    let query = format!("SELECT * FROM projects {}", sort.to_order_by_clause());
    // ...
}
```

## PgExecutor

Pool と Transaction を抽象化し、リポジトリが両方で動作できるようにする。

**設計原則**:
- sqlx の Query 型の種類ごとに Query 実行メソッドを実装する
- Pool/Transaction の分岐は `PgExecutor` 内に閉じ込め、リポジトリには露出させない
- 新しいクエリパターンが必要になった場合は `PgExecutor` にメソッドを追加する

```rust
// src/repository/executor.rs

pub enum PgExecutor {
    Pool(PgPool),
    Transaction(Arc<Mutex<Transaction<'static, Postgres>>>),
}

impl PgExecutor {
    /// 単一行を取得する（存在しない場合は None）
    pub async fn fetch_optional<'q, T>(
        &self,
        query: sqlx::query::QueryAs<'q, Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    {
        match self {
            Self::Pool(pool) => query.fetch_optional(pool).await,
            Self::Transaction(tx) => {
                let mut guard = tx.lock().await;
                query.fetch_optional(&mut **guard).await
            }
        }
    }

    // fetch_all, fetch_one_scalar, execute, ...
}
```

## リポジトリ実装

**設計原則**:
- リポジトリは `PgExecutor` を受け取る
- SQL は一度だけ記述し、`PgExecutor` のメソッドに委譲する
- リポジトリ内で Pool/Transaction の match 分岐を書かない

```rust
// src/repository/project_repo.rs

pub struct PgProjectRepository {
    executor: PgExecutor,
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError> {
        let query = sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
            .bind(id.0);

        self.executor
            .fetch_optional(query)
            .await
            .map(|row| row.map(Project::from))
            .map_err(|e| map_sqlx_error(e, "project"))
    }

    // find_all, save, exists_by_name, ...
}
```

## 集約スコープによる fetch / save 範囲の制御

複数レイヤーを持つ集約（`Trial` → `Step` → `Parameter` など）のリポジトリは、
**fetch / save が触れるレイヤーの深さを scope 引数で受け取り、その範囲だけを処理する**。
これはモデル個別の事情ではなく、リポジトリ層共通の仕様として扱う。

**scope enum の定義**

- scope enum は ports 層の該当リポジトリトレイトと同じファイルに定義する（例: `TrialScope`）
- 深さの昇順で `PartialOrd` / `Ord` を導出し、`scope < Xxx` で段階を判定できるようにする
- `Default` は最も深いバリアント（全レイヤー）とし、scope 未指定に相当する呼び出しの後方互換を保つ
- **各バリアントが含むレイヤーの説明はバリアント側にだけ書く**。
  トレイトメソッドの引数や実装側のコメントで範囲を繰り返さない

**リポジトリ実装での遵守事項**

| 操作 | scope が満たさないレイヤーに対して |
|------|--------------------------------|
| find 系 | SELECT を発行しない |
| save 系 | DELETE / INSERT / UPDATE を発行しない |

save は「渡された集約のスナップショットを正として DB との差分を洗い替える」方式のため、
scope の遵守は **部分スコープで取得した集約をそのまま save しても下位レイヤーが消えない**
ことを支える不変条件になる。scope を判定する早期リターンより前に無条件の
DELETE / UPSERT を追加しないこと。

**scope を判定する場所**

scope の分岐は「そのレイヤーを扱うかどうか」を決める呼び出し側に置き、
レイヤー単位のヘルパー関数の中でスコープ外バリアントを処理しない。

```rust
// ❌ Step を扱う関数が、Step を扱わない scope の分岐まで抱えている
async fn fetch_steps_by_trial_ids(&self, ids: &[Uuid], scope: TrialScope) -> ... {
    match scope {
        TrialScope::TrialOnly => Ok(HashMap::new()),  // この関数の責務外
        // ...
    }
}

// ✅ 呼び出し側で弾き、ヘルパーは自分が扱うレイヤーだけを見る
let steps_by_trial = if scope < TrialScope::WithSteps {
    HashMap::new()
} else {
    self.fetch_steps_by_trial_ids(&trial_ids, scope).await?
};
```

**scope の組み立て**

| 呼び出し側 | 指定方法 |
|-----------|---------|
| presentation（read） | GraphQL の selection set から動的に組み立てる |
| presentation（write） | 戻り値として選択されたレイヤーを selection set から組み立て、`return_scope` としてユースケースへ渡す |
| use_case（write） | 書き込むレイヤー（`write_scope`）を静的に指定し、find は `write_scope` と `return_scope` の深い方を使う |

**write ユースケースの戻り値スコープ**

書き込みユースケースは find した集約を加工してそのまま呼び出し元へ返すため、
**書き込みに必要なレイヤーだけで find すると、戻り値から下位レイヤーが黙って消える**。
たとえば `updateTrial` が `TrialOnly` で find した Trial をそのまま返すと、
`updateTrial { steps { id } }` は DB に Step があっても常に `steps: []` を返す。

そのため write ユースケースは戻り値に必要なレイヤーを `return_scope` として受け取り、
find は `read_scope_for_write(write_scope, return_scope)`（= 深い方）で行う。
save は `write_scope` のまま行うため、戻り値のために読み込んだ下位レイヤーが
DB へ書き戻されることはない。

```rust
// ❌ 書き込み範囲だけで find すると戻り値から Step/Parameter が消える
let trial = repo.find_by_id(&trial_id, TrialScope::TrialOnly).await?;
// ...
Ok(updated)

// ✅ 戻り値に必要なレイヤーまで find し、save は書き込み範囲のまま
let trial = repo
    .find_by_id(&trial_id, read_scope_for_write(TrialScope::TrialOnly, return_scope))
    .await?;
// ...
save_trial(uow, &updated, TrialScope::TrialOnly).await?;
Ok(updated)
```

**例外: 新規作成した実体のみを返す write ユースケース**

`return_scope` が必要なのは **戻り値に既存の集約（DB に下位レイヤーが存在しうる実体）を含む**
場合に限る。新規作成した実体だけを返すユースケースは、その実体の下位レイヤーが
**作成直後で元から空**であり、DB の実データと一致するため `return_scope` を受け取らない。

| ユースケース | 戻り値 | `return_scope` | 理由 |
|-------------|--------|---------------|------|
| `create_trial` | 作成直後の Trial | 不要 | 新規 Trial は Step を持たない |
| `add_step` | 追加直後の Step | 不要 | 新規 Step は Parameter を持たない |
| `update_trial` / `complete_trial` 等 | 既存の Trial | **必要** | DB に Step/Parameter が存在しうる |

この例外は「作成直後だから空」であることに依存している。既存の集約を返すユースケースを
`add_step` を前例にして `return_scope` なしで追加すると、戻り値から下位レイヤーが黙って
消える退行がそのまま再発する。

**不変条件: save scope ≤ find scope**

同一の集約に対して、**save の scope は必ずその集約を find した scope 以下**でなければならない。
find より深い scope で save すると、find していない（＝集約上は空の）レイヤーが
「削除された」と解釈され、洗い替えの差分削除で DB 上の実データが消える。

`read_scope_for_write(write_scope, return_scope)` が常に `write_scope` 以上を返し、
save は `write_scope` のまま行うため、現在の実装ではこの不変条件は構造的に満たされている。
scope の組み立てを変更する際はこの関係を崩さないこと。

## スキーマ命名規約

DB のスキーマ名はプロジェクトのルールとして以下に統一する。PostgreSQL がデフォルトで付与する名前は
リネームせずそのまま使い、明示的に付与するのはインデックスのみとする。

| 対象 | 命名 | 備考 |
|------|------|------|
| テーブル名 | 複数形の snake_case | `projects` / `trials` / `steps` / `parameters` |
| エンティティ名（Rust） | テーブル名の単数形 | `project` / `trial` / `step` / `parameter` |
| 主キー | `{table}_pkey` | PostgreSQL のデフォルト。リネームしない |
| 一意制約 | `{table}_{columns}_key` | PostgreSQL のデフォルト。リネームしない |
| 外部キー | `{table}_{column}_fkey` | PostgreSQL のデフォルト。リネームしない。`{table}` は常に **参照する側** |
| インデックス | `idx_{table}_{columns}` | `CREATE INDEX` で明示的に付与する |

**一意インデックス**は `CREATE UNIQUE INDEX` で一意性を表現する。`uq_` / `unique_` のような
接頭辞や `_unique` / `_idx` のような接尾辞は使用しない。

制約名の `{table}_` 接頭辞から外部キーの向きを判定するため、**既存テーブル名の複数形で始まる
テーブル名**（例: `steps` が存在する状態での `steps_archive`）は作らない。

### Rust 側の定義

上記の接頭辞・接尾辞は `backend/src/repository/naming_conventions.rs` に定数として定義する。
制約名からテーブル接頭辞を取り除くヘルパーも同モジュールに集約しており、
テーブル名が複数形であることを前提に `{entity}s_` のみを対象とする。

`map_sqlx_error` はこの規約を前提に制約名からエンティティ名・フィールド名を復元する。
**規約から外れた名前を付けてもコンパイルは通る**ため、リネームするとエラー分類だけが静かに劣化する
（一意制約違反が `Conflict` の `field` に制約名がそのまま現れる、外部キー違反の向き判定が崩れる等）。
マイグレーションで制約名を変更する場合は `naming_conventions.rs` とそのテストも併せて更新すること。

## アンチパターン

```rust
// ❌ リポジトリ内でビジネスロジック
if name.is_empty() { return Err(Error::EmptyName); }
let project = Project::new(name, ...);

// ❌ ドメインモデルに FromRow
#[derive(FromRow)]  // ドメイン層がsqlxに依存
pub struct Project { ... }

// ❌ SQLインジェクション
let query = format!("SELECT * FROM projects WHERE name = '{}'", name);

// ❌ トランザクション管理
let mut tx = self.pool.begin().await?;

// ✅ 受け取ったモデルを保存、プレースホルダー使用、単一操作のみ
```

## チェックリスト

- [ ] portsトレイトを実装
- [ ] ビジネスロジックを含まない
- [ ] DBモデルとドメインモデルが分離
- [ ] プレースホルダー使用（SQLインジェクション対策）
- [ ] UPSERT（ON CONFLICT）で冪等性確保
- [ ] sqlx エラーは `map_sqlx_error` で変換する（`RepositoryError::Internal` に畳まない）
- [ ] 複数レイヤーを持つ集約では、scope が満たさないレイヤーのテーブルにクエリを発行していない
- [ ] scope の分岐は呼び出し側にあり、レイヤー単位のヘルパーがスコープ外バリアントを処理していない
- [ ] レイヤー単位のヘルパーはスコープ外で呼ばれたことを `debug_assert!` で検知できる
- [ ] 戻り値に既存の集約を含む write ユースケースは `return_scope` を受け取り、find を `read_scope_for_write` で拡張している（新規作成した実体のみを返す場合は下位レイヤーが元から空のため不要）
- [ ] 同一集約に対する save の scope が find の scope 以下になっている
- [ ] テスト用モックの save は、新規挿入パスでも scope 外レイヤーを保存しない（本番より寛容にしない）
- [ ] テーブル名は複数形の snake_case、エンティティ名はその単数形
- [ ] 主キー・一意制約・外部キーは PostgreSQL のデフォルト名をリネームしない
- [ ] インデックスは `idx_{table}_{columns}` で明示的に命名し、一意性は `CREATE UNIQUE INDEX` で表現する
