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

/// 試行（Trial）- 集約ルート
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    status: TrialStatus,
    memo: Option<String>,
    steps: Vec<Step>,
}

impl Trial {
    /// 新しい Trial を作成する（Steps は空）
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            id: TrialId::new(),
            project_id,
            status: TrialStatus::InProgress,
            memo: None,
            steps: Vec::new(),
        }
    }

    /// DB などから Trial を復元する
    pub fn from_raw(
        id: TrialId,
        project_id: ProjectId,
        status: TrialStatus,
        memo: Option<String>,
        steps: Vec<Step>,
    ) -> Self {
        Self {
            id,
            project_id,
            status,
            memo,
            steps,
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

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// 次の Step の position を取得
    pub fn next_step_position(&self) -> u8 {
        self.steps
            .iter()
            .map(|s| s.position())
            .max()
            .map(|max| max + 1)
            .unwrap_or(0)
    }

    /// Step を追加する
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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

    #[test]
    fn test_trial_new_has_empty_steps() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id);
        assert!(trial.steps().is_empty());
    }

    #[test]
    fn test_trial_next_step_position_empty() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id);
        assert_eq!(trial.next_step_position(), 0);
    }

    #[test]
    fn test_trial_next_step_position_with_steps() {
        let project_id = ProjectId::new();
        let now = Utc::now();
        let step1 = Step::new(0, now);
        let step2 = Step::new(2, now);
        let trial = Trial::from_raw(
            TrialId::new(),
            project_id,
            TrialStatus::InProgress,
            None,
            vec![step1, step2],
        );
        assert_eq!(trial.next_step_position(), 3);
    }

    #[test]
    fn test_trial_add_step() {
        let project_id = ProjectId::new();
        let mut trial = Trial::new(project_id);
        let now = Utc::now();
        let step = Step::new(0, now);
        trial.add_step(step);
        assert_eq!(trial.steps().len(), 1);
    }
}
