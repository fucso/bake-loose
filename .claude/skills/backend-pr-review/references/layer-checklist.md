# レイヤー別レビューチェックリスト

各レイヤーの詳細なレビュー観点。

## 目次

1. [Domain層](#domain層)
2. [Ports層](#ports層)
3. [Repository層](#repository層)
4. [UseCase層](#usecase層)
5. [Presentation層](#presentation層)
6. [テスト](#テスト)

---

## Domain層

### ファイル配置

```
backend/src/domain/
├── models/          # Project, Trial, ...
└── actions/         # 1アクション1ファイル
    └── {entity}/
        └── {action}.rs
```

### チェックリスト

- [ ] 外部クレートへの依存がない（uuid, serdeのみ許可）
- [ ] I/O操作を行っていない
- [ ] IDはNewTypeパターン（`pub struct ProjectId(pub Uuid)`）
- [ ] 1アクション1ファイル
- [ ] validate / execute / run が分離されている
- [ ] エラー型は種類のみ（メッセージを含まない）

### アクション構造

```rust
// 正しいパターン
pub struct Command { pub name: String }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyName,  // メッセージなし
    NameTooLong { max: usize, actual: usize },
}

pub fn validate(command: &Command) -> Result<(), Error> { ... }
pub fn execute(command: Command) -> Entity { ... }
pub fn run(command: Command) -> Result<Entity, Error> {
    validate(&command)?;
    Ok(execute(command))
}
```

### アンチパターン

```rust
// ❌ 外部依存
use sqlx::PgPool;

// ❌ エラーにメッセージ
pub enum Error { EmptyName { message: String } }

// ❌ 可変参照による状態変更
pub fn update_name(&mut self, name: String) { self.name = name; }
```

---

## Ports層

### ファイル配置

```
backend/src/ports/
├── project_repository.rs
├── unit_of_work.rs
├── sort.rs
└── error.rs
```

### チェックリスト

- [ ] domain層にのみ依存
- [ ] 外部クレートへの依存がない
- [ ] `async_trait` + `Send + Sync` バウンド
- [ ] すべてのメソッドが `Result` 型を返す
- [ ] ドメインモデルの型のみを使用

### トレイト定義

```rust
#[async_trait::async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, RepositoryError>;
    async fn find_all(&self, sort: ProjectSort) -> Result<Vec<Project>, RepositoryError>;
    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError>;
    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}
```

### UnitOfWork

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type ProjectRepo: ProjectRepository;

    fn project_repository(&mut self) -> Self::ProjectRepo;  // 毎回新しいインスタンス
    async fn begin(&mut self) -> Result<(), RepositoryError>;
    async fn commit(&mut self) -> Result<(), RepositoryError>;
    async fn rollback(&mut self) -> Result<(), RepositoryError>;
}
```

### アンチパターン

```rust
// ❌ SQL固有の構文
async fn find_by_sql(&self, where_clause: &str) -> ...;

// ❌ DB固有の型
use sqlx::PgPool;
```

---

## Repository層

### ファイル配置

```
backend/src/repository/
├── executor.rs         # PgExecutor
├── pg_unit_of_work.rs  # PgUnitOfWork
├── project_repo.rs
└── models/
    └── project_row.rs  # DBモデル
```

### チェックリスト

- [ ] portsトレイトを実装
- [ ] ビジネスロジックを含まない
- [ ] DBモデルとドメインモデルが分離（`ProjectRow` → `Project`）
- [ ] プレースホルダー使用（SQLインジェクション対策）
- [ ] UPSERT（`ON CONFLICT`）で冪等性確保
- [ ] `PgExecutor` を使用（Pool/Transactionの分岐を隠蔽）

### PgExecutor

```rust
pub enum PgExecutor {
    Pool(PgPool),
    Transaction(Arc<Mutex<Transaction<'static, Postgres>>>),
}

impl PgExecutor {
    pub async fn fetch_optional<T>(&self, query: ...) -> Result<Option<T>, sqlx::Error>;
    pub async fn fetch_all<T>(&self, query: ...) -> Result<Vec<T>, sqlx::Error>;
    pub async fn fetch_one_scalar<T>(&self, query: ...) -> Result<T, sqlx::Error>;
    pub async fn execute(&self, query: ...) -> Result<PgQueryResult, sqlx::Error>;
}
```

### UPSERT パターン

```sql
INSERT INTO projects (id, name, created_at, updated_at)
VALUES ($1, $2, NOW(), NOW())
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    updated_at = NOW()
```

### アンチパターン

```rust
// ❌ ビジネスロジック
if name.is_empty() { return Err(Error::EmptyName); }

// ❌ ドメインモデルにFromRow
#[derive(FromRow)]
pub struct Project { ... }

// ❌ SQLインジェクション
let query = format!("SELECT * FROM projects WHERE name = '{}'", name);

// ❌ リポジトリ内でトランザクション管理
let mut tx = self.pool.begin().await?;
```

---

## UseCase層

### ファイル配置

```
backend/src/use_case/
└── {entity}/
    └── {use_case}.rs  # 1ユースケース1ファイル
```

### チェックリスト

- [ ] domain層とports層にのみ依存
- [ ] ドメインアクションを呼び出している（直接ロジック実装していない）
- [ ] UnitOfWork経由で永続化
- [ ] DB検証はドメインアクション実行前
- [ ] 書き込み操作では `begin()` でトランザクション開始
- [ ] 成功後に `commit()` を呼んでいる
- [ ] エラー時は `rollback()` を呼んでいる

### 実装パターン

```rust
pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Entity, Error> {
    // 1. トランザクション開始
    uow.begin().await.map_err(|e| Error::Infrastructure(...))?;

    // 2. DB検証（先に行う）
    if uow.repository().exists_by_name(&input.name).await? {
        let _ = uow.rollback().await;
        return Err(Error::DuplicateName);
    }

    // 3. ドメインアクション実行
    let entity = match domain_action::run(command) {
        Ok(e) => e,
        Err(e) => {
            let _ = uow.rollback().await;
            return Err(Error::Domain(e));
        }
    };

    // 4. 永続化
    if let Err(e) = uow.repository().save(&entity).await {
        let _ = uow.rollback().await;
        return Err(Error::Infrastructure(...));
    }

    // 5. コミット
    uow.commit().await.map_err(|e| Error::Infrastructure(...))?;

    Ok(entity)
}
```

### 読み取り専用

```rust
// begin() 不要
pub async fn execute<U: UnitOfWork>(uow: &mut U) -> Result<Vec<Entity>, Error> {
    uow.repository().find_all(sort).await.map_err(...)
}
```

### アンチパターン

```rust
// ❌ ユースケースでバリデーション
if input.name.is_empty() { return Err(Error::EmptyName); }

// ❌ SQL直接記述
sqlx::query("INSERT INTO ...").execute(pool).await?;

// ❌ 検証の順序が不適切
let entity = domain_action::run(command)?;
if repository.exists_by_name(&entity.name()).await? { ... }  // 後から検証

// ❌ begin() なしで書き込み
uow.repository().save(&entity).await?;
uow.commit().await?;
```

---

## Presentation層

### ファイル配置

```
backend/src/presentation/graphql/
├── schema.rs
├── context.rs
├── error.rs
├── types/
│   └── project.rs  # ラッパー型 + InputObject
├── query/
│   └── project.rs
└── mutation/
    └── project.rs
```

### チェックリスト

- [ ] リゾルバーはユースケース呼び出しのみ
- [ ] ドメインモデルを直接公開していない（ラッパー型使用）
- [ ] 各層のエラーにメッセージが含まれていない
- [ ] リポジトリを直接参照していない
- [ ] 内部エラーの詳細をクライアントに露出していない

### ラッパー型

```rust
pub struct Project(pub DomainProject);

#[Object]
impl Project {
    async fn id(&self) -> ID { ID(self.0.id().0.to_string()) }
    async fn name(&self) -> &str { self.0.name() }
}

impl From<DomainProject> for Project {
    fn from(p: DomainProject) -> Self { Self(p) }
}
```

### エラー変換

```rust
pub trait UserFacingError {
    fn to_user_facing(&self) -> GraphQLError;
}

impl UserFacingError for use_case::Error {
    fn to_user_facing(&self) -> GraphQLError {
        match self {
            Error::Domain(e) => match e {
                DomainError::EmptyName =>
                    GraphQLError::new("プロジェクト名を入力してください", "VALIDATION_ERROR"),
                // ...
            },
            Error::DuplicateName =>
                GraphQLError::new("同じ名前が既に存在します", "DUPLICATE_ERROR"),
            Error::Infrastructure(e) => {
                log::error!("Infrastructure error: {}", e);
                GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR")
            }
        }
    }
}
```

### エラーコード規約

| コード | 用途 |
|-------|------|
| `VALIDATION_ERROR` | 入力バリデーション |
| `NOT_FOUND` | リソース不存在 |
| `DUPLICATE_ERROR` | 重複 |
| `INTERNAL_ERROR` | 内部エラー（詳細は隠す） |

### アンチパターン

```rust
// ❌ リゾルバーでビジネスロジック
if input.name.is_empty() { return Err(...); }

// ❌ ドメインモデルに直接GraphQLアノテーション
#[derive(SimpleObject)]
pub struct Project { ... }

// ❌ 各層でメッセージ定義
pub enum Error { EmptyName { message: String } }

// ❌ リポジトリ直接参照
let repo = ctx.data::<ProjectRepository>()?;
```

---

## テスト

### 配置ルール

| レイヤー | テスト配置 |
|---------|----------|
| domain | `src/domain/actions/{entity}/{action}.rs` 内 `#[cfg(test)] mod tests` |
| ports | なし（トレイト定義のみ） |
| use_case | `src/use_case/{entity}/{use_case}.rs` 内 `#[cfg(test)] mod tests` |
| repository | `src/repository/{entity}_repo.rs` 内 `#[sqlx::test]` |
| presentation | `tests/graphql/{entity}/{operation}.rs` |

### MockUnitOfWork

```rust
// src/use_case/test/mock_unit_of_work.rs
pub struct MockUnitOfWork {
    projects: Arc<Mutex<Vec<Project>>>,
    transaction_started: bool,
}

impl UnitOfWork for MockUnitOfWork {
    fn project_repository(&mut self) -> MockProjectRepository { ... }
    async fn begin(&mut self) -> Result<(), RepositoryError> { ... }
    async fn commit(&mut self) -> Result<(), RepositoryError> { ... }
    async fn rollback(&mut self) -> Result<(), RepositoryError> { ... }
}
```

### GraphQL統合テスト

```rust
// tests/graphql/schema.rs
pub async fn execute_graphql(pool: PgPool, query: &str) -> serde_json::Value;
pub async fn execute_graphql_with_errors(pool: PgPool, query: &str) -> Response;

// tests/graphql/projects/create.rs
#[sqlx::test(migrations = "./migrations")]
async fn test_creates_project_successfully(pool: PgPool) {
    let data = execute_graphql(pool, &mutation).await;
    assert_eq!(data["createProject"]["name"], "新規プロジェクト");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_error_for_empty_name(pool: PgPool) {
    let response = execute_graphql_with_errors(pool, &mutation).await;
    assert_eq!(response.errors[0].extensions["code"], "VALIDATION_ERROR");
}
```
