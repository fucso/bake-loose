//! Step ドメインモデル

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::parameter::{Parameter, ParameterId};
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    trial_id: TrialId,
    name: String,
    position: i32,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    parameters: Vec<Parameter>,
}

impl Step {
    /// 新しいStepを作成する（ID は自動生成、parameters は空）
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
            parameters: Vec::new(),
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
        parameters: Vec<Parameter>,
    ) -> Self {
        Self {
            id,
            trial_id,
            name,
            position,
            started_at,
            completed_at,
            parameters,
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

    /// name を設定する
    pub fn set_name(&mut self, name: String) {
        self.name = name;
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

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Parameter を追加する
    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(parameter);
    }

    /// Parameter を削除する（該当IDが存在しない場合は何もしない）
    pub fn remove_parameter(&mut self, parameter_id: &ParameterId) {
        self.parameters.retain(|p| p.id() != parameter_id);
    }

    /// Step を完了状態にする
    ///
    /// completed_at が未指定の場合は Utc::now() を採用する
    pub fn complete(&mut self, completed_at: Option<DateTime<Utc>>) {
        self.completed_at = Some(completed_at.unwrap_or_else(Utc::now));
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
    fn test_set_name_updates_name() {
        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        step.set_name("発酵".to_string());
        assert_eq!(step.name(), "発酵");
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
    fn test_complete_uses_specified_completed_at() {
        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        let completed_at = Utc::now();

        step.complete(Some(completed_at));

        assert_eq!(step.completed_at(), Some(&completed_at));
        assert!(step.is_completed());
    }

    #[test]
    fn test_complete_defaults_completed_at_to_now_when_unspecified() {
        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);

        step.complete(None);

        assert!(step.completed_at().is_some());
        assert!(step.is_completed());
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
            Vec::new(),
        );
        assert!(completed.is_completed());
    }

    #[test]
    fn test_new_step_has_no_parameters() {
        let step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_add_parameter_appends_to_parameters() {
        use super::super::parameter::{Parameter, ParameterContent};

        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        let parameter = Parameter::new(
            step.id().clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();

        step.add_parameter(parameter);

        assert_eq!(step.parameters().len(), 1);
        assert_eq!(step.parameters()[0].id(), &parameter_id);
    }

    #[test]
    fn test_remove_parameter_removes_matching_id() {
        use super::super::parameter::{Parameter, ParameterContent};

        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        let parameter = Parameter::new(
            step.id().clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        assert_eq!(step.parameters().len(), 1);

        step.remove_parameter(&parameter_id);

        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_remove_parameter_is_noop_when_id_not_found() {
        use super::super::parameter::ParameterId;

        let mut step = Step::new(TrialId::new(), "こね".to_string(), 0, None);
        step.remove_parameter(&ParameterId::new());
        assert!(step.parameters().is_empty());
    }
}
