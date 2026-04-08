//! TrialRepository トレイト

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;
use crate::ports::sort::{Sort, SortColumn};

/// Trial のソート可能カラム
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrialSortColumn {
    #[default]
    CreatedAt,
    UpdatedAt,
}

impl SortColumn for TrialSortColumn {
    fn as_sql_column(&self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

/// Trial 一覧のソート条件
pub type TrialSort = Sort<TrialSortColumn>;

/// Trial リポジトリのトレイト
///
/// Trial を aggregate root として Steps と Parameters を含めて操作する。
#[async_trait::async_trait]
pub trait TrialRepository: Send + Sync {
    /// ID で Trial を取得する（Steps と Parameters を含む）
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError>;

    /// プロジェクトに紐づく Trial 一覧を取得する
    async fn find_by_project_id(
        &self,
        project_id: &ProjectId,
        sort: TrialSort,
    ) -> Result<Vec<Trial>, RepositoryError>;

    /// Trial を保存する（新規作成または更新）
    ///
    /// Steps と Parameters も含めて UPSERT する。
    async fn save(&self, trial: &Trial) -> Result<(), RepositoryError>;

    /// Trial を削除する（Steps と Parameters も CASCADE 削除）
    async fn delete(&self, id: &TrialId) -> Result<(), RepositoryError>;
}
