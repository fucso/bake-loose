//! PgUnitOfWork 実装
//!
//! UnitOfWork トレイトの PostgreSQL 実装。

use async_trait::async_trait;
use sqlx::PgPool;

use crate::ports::error::RepositoryError;
use crate::ports::UnitOfWork;

use super::project_repo::PgProjectRepository;

/// PostgreSQL 用の UnitOfWork 実装
#[derive(Clone)]
pub struct PgUnitOfWork {
    project_repo: PgProjectRepository,
}

impl PgUnitOfWork {
    /// 新しい PgUnitOfWork を作成する
    pub fn new(pool: PgPool) -> Self {
        Self {
            project_repo: PgProjectRepository::new(pool),
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    type ProjectRepo = PgProjectRepository;

    fn project_repository(&self) -> &Self::ProjectRepo {
        &self.project_repo
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        // 今回は読み取り専用のため何もしない
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }
}
