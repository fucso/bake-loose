//! Trial ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::step::{Step, StepId};
use crate::domain::models::project::ProjectId;
use crate::domain::timezone::JstDateTime;

/// TrialID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrialId(pub Uuid);

impl TrialId {
    /// 新しいTrialIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrialId {
    fn default() -> Self {
        Self::new()
    }
}

/// Trial の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    InProgress,
    Completed,
}

/// Trial（Project に紐づく試行。Step を含む aggregate root）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    name: Option<String>,
    memo: Option<String>,
    status: TrialStatus,
    completed_at: Option<JstDateTime>,
    steps: Vec<Step>,
}

impl Trial {
    /// 新しいTrialを作成する（ID は自動生成、status は InProgress）
    pub fn new(project_id: ProjectId, name: Option<String>, memo: Option<String>) -> Self {
        Self {
            id: TrialId::new(),
            project_id,
            name,
            memo,
            status: TrialStatus::InProgress,
            completed_at: None,
            steps: Vec::new(),
        }
    }

    /// 生データからTrialを構築する
    #[allow(clippy::too_many_arguments)]
    pub fn from_raw(
        id: TrialId,
        project_id: ProjectId,
        name: Option<String>,
        memo: Option<String>,
        status: TrialStatus,
        completed_at: Option<JstDateTime>,
        steps: Vec<Step>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            memo,
            status,
            completed_at,
            steps,
        }
    }

    pub fn id(&self) -> &TrialId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn memo(&self) -> Option<&str> {
        self.memo.as_deref()
    }

    pub fn status(&self) -> &TrialStatus {
        &self.status
    }

    /// 完了日時（JST）
    pub fn completed_at(&self) -> Option<&JstDateTime> {
        self.completed_at.as_ref()
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Step を ID で取得する
    pub fn step(&self, id: &StepId) -> Option<&Step> {
        self.steps.iter().find(|step| step.id() == id)
    }

    /// Step を可変参照として取得・変更するためのアクセサ
    pub fn steps_mut(&mut self) -> &mut Vec<Step> {
        &mut self.steps
    }

    /// Step を追加する
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// name を設定・クリアする
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// memo を設定・クリアする
    pub fn set_memo(&mut self, memo: Option<String>) {
        self.memo = memo;
    }

    /// Trial を完了状態にする
    ///
    /// completed_at が未指定の場合は現在時刻を採用する
    pub fn complete(&mut self, completed_at: Option<JstDateTime>) {
        self.status = TrialStatus::Completed;
        self.completed_at = Some(completed_at.unwrap_or_else(JstDateTime::now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_id_new_generates_unique_ids() {
        let id1 = TrialId::new();
        let id2 = TrialId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trial_new_creates_in_progress_with_no_steps() {
        let trial = Trial::new(ProjectId::new(), Some("焼成温度検証".to_string()), None);
        assert_eq!(trial.status(), &TrialStatus::InProgress);
        assert!(trial.steps().is_empty());
        assert_eq!(trial.name(), Some("焼成温度検証"));
        assert_eq!(trial.memo(), None);
    }

    #[test]
    fn test_complete_transitions_status_to_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        assert_eq!(trial.status(), &TrialStatus::InProgress);

        trial.complete(None);
        assert_eq!(trial.status(), &TrialStatus::Completed);
    }

    #[test]
    fn test_complete_uses_specified_completed_at() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let completed_at = JstDateTime::now();

        trial.complete(Some(completed_at));

        assert_eq!(trial.completed_at(), Some(&completed_at));
    }

    #[test]
    fn test_complete_defaults_completed_at_to_now_when_unspecified() {
        let mut trial = Trial::new(ProjectId::new(), None, None);

        trial.complete(None);

        assert!(trial.completed_at().is_some());
    }

    #[test]
    fn test_add_step_appends_to_steps() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();

        trial.add_step(step);

        assert_eq!(trial.steps().len(), 1);
        assert_eq!(trial.steps()[0].id(), &step_id);
    }

    #[test]
    fn test_step_returns_matching_step_by_id() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);

        assert_eq!(trial.step(&step_id).unwrap().id(), &step_id);
        assert!(trial.step(&StepId::new()).is_none());
    }

    #[test]
    fn test_set_name_can_set_and_clear() {
        let mut trial = Trial::new(ProjectId::new(), Some("元の名前".to_string()), None);

        trial.set_name(Some("新しい名前".to_string()));
        assert_eq!(trial.name(), Some("新しい名前"));

        trial.set_name(None);
        assert_eq!(trial.name(), None);
    }

    #[test]
    fn test_set_memo_can_set_and_clear() {
        let mut trial = Trial::new(ProjectId::new(), None, Some("元のメモ".to_string()));

        trial.set_memo(Some("新しいメモ".to_string()));
        assert_eq!(trial.memo(), Some("新しいメモ"));

        trial.set_memo(None);
        assert_eq!(trial.memo(), None);
    }

    #[test]
    fn test_steps_mut_allows_mutating_step_by_id() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        trial.add_step(Step::new(trial.id().clone(), "こね".to_string(), 0, None));
        let target_step = Step::new(trial.id().clone(), "発酵".to_string(), 1, None);
        let target_id = target_step.id().clone();
        trial.add_step(target_step);

        let new_started_at = crate::domain::timezone::JstDateTime::now();
        let step = trial
            .steps_mut()
            .iter_mut()
            .find(|s| s.id() == &target_id)
            .expect("target step must exist");
        step.start(Some(new_started_at));

        let updated = trial.steps().iter().find(|s| s.id() == &target_id).unwrap();
        assert_eq!(updated.started_at(), Some(&new_started_at));
    }
}
