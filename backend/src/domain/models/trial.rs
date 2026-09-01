//! Trial ドメインモデル

mod parameter;
mod step;

pub use parameter::*;
pub use step::*;

use super::project::ProjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trial の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrialId(pub Uuid);

impl TrialId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrialId {
    fn default() -> Self {
        Self::new()
    }
}

/// 試行のステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    InProgress,
    Completed,
}

/// 試行（Trial）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    status: TrialStatus,
    memo: Option<String>,
}

impl Trial {
    /// 新しい Trial を作成する
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            id: TrialId::new(),
            project_id,
            status: TrialStatus::InProgress,
            memo: None,
        }
    }

    /// DB などから Trial を復元する
    pub fn from_raw(
        id: TrialId,
        project_id: ProjectId,
        status: TrialStatus,
        memo: Option<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            status,
            memo,
        }
    }

    // Getters
    pub fn id(&self) -> &TrialId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn status(&self) -> TrialStatus {
        self.status
    }

    pub fn memo(&self) -> Option<&str> {
        self.memo.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_id_generates_unique() {
        let id1 = TrialId::new();
        let id2 = TrialId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trial_new_creates_with_in_progress_status() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id.clone());
        assert_eq!(trial.status(), TrialStatus::InProgress);
        assert_eq!(trial.project_id(), &project_id);
        assert!(trial.memo().is_none());
    }
}
