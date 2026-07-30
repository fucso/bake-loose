//! Step ドメインモデル

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::trial::TrialId;

/// StepID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    /// 新しいStepIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

/// Step（Trial に紐づく試行の1工程）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    trial_id: TrialId,
    name: String,
    position: i32,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl Step {
    /// 新しいStepを作成する（ID は自動生成）
    ///
    /// started_at が未指定の場合は Utc::now() を採用する
    pub fn new(
        trial_id: TrialId,
        name: String,
        position: i32,
        started_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: StepId::new(),
            trial_id,
            name,
            position,
            started_at: Some(started_at.unwrap_or_else(Utc::now)),
            completed_at: None,
        }
    }

    /// 生データからStepを構築する
    pub fn from_raw(
        id: StepId,
        trial_id: TrialId,
        name: String,
        position: i32,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            trial_id,
            name,
            position,
            started_at,
            completed_at,
        }
    }

    pub fn id(&self) -> &StepId {
        &self.id
    }

    pub fn trial_id(&self) -> &TrialId {
        &self.trial_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> i32 {
        self.position
    }

    pub fn started_at(&self) -> Option<&DateTime<Utc>> {
        self.started_at.as_ref()
    }

    pub fn completed_at(&self) -> Option<&DateTime<Utc>> {
        self.completed_at.as_ref()
    }

    /// 完了済みかどうかを判定する（completed_at の有無で判定）
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// started_at を設定・クリアする
    pub fn set_started_at(&mut self, started_at: Option<DateTime<Utc>>) {
        self.started_at = started_at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_id_new_generates_unique_ids() {
        let id1 = StepId::new();
        let id2 = StepId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_step_new_defaults_started_at_to_now_when_unspecified() {
        let step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        assert!(step.started_at().is_some());
        assert!(step.completed_at().is_none());
        assert!(!step.is_completed());
    }

    #[test]
    fn test_step_new_uses_specified_started_at() {
        let started_at = Utc::now() - chrono::Duration::hours(1);
        let step = Step::new(TrialId::new(), "こね".to_string(), 0, Some(started_at));
        assert_eq!(step.started_at(), Some(&started_at));
    }

    #[test]
    fn test_set_started_at_can_set_and_clear() {
        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);

        let new_started_at = Utc::now();
        step.set_started_at(Some(new_started_at));
        assert_eq!(step.started_at(), Some(&new_started_at));

        step.set_started_at(None);
        assert_eq!(step.started_at(), None);
    }

    #[test]
    fn test_is_completed_reflects_completed_at() {
        let trial_id = TrialId::new();
        let in_progress = Step::new(trial_id.clone(), "こね".to_string(), 0, None);
        assert!(!in_progress.is_completed());

        let completed = Step::from_raw(
            StepId::new(),
            trial_id,
            "こね".to_string(),
            0,
            Some(Utc::now()),
            Some(Utc::now()),
        );
        assert!(completed.is_completed());
    }
}
