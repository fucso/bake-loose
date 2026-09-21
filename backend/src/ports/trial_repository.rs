//! TrialRepository トレイト

use crate::domain::models::project::ProjectId;
use crate::domain::models::trial::{Trial, TrialId};
use crate::ports::error::RepositoryError;

#[async_trait::async_trait]
pub trait TrialRepository: Send + Sync {
    async fn find_by_id(&self, id: &TrialId) -> Result<Option<Trial>, RepositoryError>;

    async fn find_all_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Trial>, RepositoryError>;

    /// Trialを保存（新規作成または更新）する
    async fn save(&self, trial: &Trial) -> Result<(), RepositoryError>;
}
