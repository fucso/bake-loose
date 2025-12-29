# Use Case Layer

ユースケース層はドメインアクションを組み合わせてビジネスフローを実現するオーケストレーション層。

## 基本原則

**依存先**: domain層、ports層のみ
**禁止**: repository層、presentation層、infrastructure層への直接依存

**やること**:
- ドメインアクションの呼び出し
- UnitOfWork経由でのリポジトリアクセス
- DB問い合わせが必要な検証
- トランザクション境界の管理

**やらないこと**:
- ビジネスロジックの実装
- 直接的なDB操作
- ユーザー向けメッセージの生成
- SQLの記述
- 個別のリポジトリを直接引数として受け取る

## ファイル配置

```
backend/src/use_case/
├── project/
│   ├── create_project.rs
│   └── ...
├── trial/
└── ...
```

**1ユースケース1ファイル**。ファイル名はドメインアクションと同じ名前。

## 実装パターン

```rust
// src/use_case/project/create_project.rs

use crate::domain::actions::project::create_project;
use crate::ports::unit_of_work::UnitOfWork;

#[derive(Debug)]
pub enum Error {
    Domain(create_project::Error),
    DuplicateName,
    Infrastructure(String),
}

pub struct Input {
    pub name: String,
    pub description: Option<String>,
}

pub async fn execute<U: UnitOfWork>(uow: &mut U, input: Input) -> Result<Project, Error> {
    // 1. DB問い合わせが必要な検証（先に行う）
    if uow.project_repository().exists_by_name(&input.name).await
        .map_err(|e| Error::Infrastructure(e.to_string()))? {
        return Err(Error::DuplicateName);
    }

    // 2. ドメインアクション実行
    let command = create_project::Command { name: input.name, ... };
    let project = create_project::run(command).map_err(Error::Domain)?;

    // 3. 永続化
    uow.project_repository().save(&project).await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;

    // 4. コミット
    uow.commit().await.map_err(|e| Error::Infrastructure(e.to_string()))?;

    Ok(project)
}
```

## エラー型

| 種類 | 発生源 |
|------|--------|
| Domain | ドメインアクションのvalidate |
| ビジネスルール | ユースケース層の検証（重複など） |
| Infrastructure | ports実装のエラー |

## アンチパターン

```rust
// ❌ ユースケースでバリデーション
if input.name.is_empty() { return Err(Error::EmptyName); }

// ❌ SQL直接記述
sqlx::query("INSERT INTO ...").execute(pool).await?;

// ❌ 検証の順序が不適切（ドメインアクション後にDB検証）
let project = create_project::run(command)?;
if repository.exists_by_name(&project.name()).await? { ... }

// ❌ リポジトリを個別に引数で受け取る
pub async fn execute(repo: &impl ProjectRepository, input: Input) -> Result<...> { ... }

// ✅ ドメインアクションに委譲、UnitOfWork経由、DB検証を先に
pub async fn execute<U: UnitOfWork>(uow: &U, input: Input) -> Result<...> { ... }
```

## チェックリスト

- [ ] domain層とports層にのみ依存
- [ ] ドメインアクションを呼び出している（直接ロジック実装していない）
- [ ] UnitOfWork経由で永続化
- [ ] DB検証はドメインアクション実行前
- [ ] 成功後に`commit()`を呼んでいる
