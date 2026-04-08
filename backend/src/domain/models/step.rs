//! Step ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterId};
use crate::domain::models::trial::TrialId;

/// ステップID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    /// 新しいステップIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

/// ステップ（Trial の工程単位）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    trial_id: TrialId,
    name: String,
    position: i16,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    parameters: Vec<Parameter>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Step {
    /// 新しいステップを作成する（ID は自動生成）
    pub fn new(trial_id: TrialId, name: String, position: i16) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: StepId::new(),
            trial_id,
            name,
            position,
            started_at: None,
            completed_at: None,
            parameters: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 生データからステップを構築する
    #[allow(clippy::too_many_arguments)]
    pub fn from_raw(
        id: StepId,
        trial_id: TrialId,
        name: String,
        position: i16,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        parameters: Vec<Parameter>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            trial_id,
            name,
            position,
            started_at,
            completed_at,
            parameters,
            created_at,
            updated_at,
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

    pub fn position(&self) -> i16 {
        self.position
    }

    pub fn started_at(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.started_at.as_ref()
    }

    pub fn completed_at(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.completed_at.as_ref()
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.updated_at
    }

    /// Step を開始状態にする
    pub fn start(&mut self, at: DateTime<Utc>) {
        self.started_at = Some(at);
        self.updated_at = Utc::now();
    }

    /// Parameter を追加する
    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(parameter);
        self.updated_at = Utc::now();
    }

    /// Step の名前を変更する
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Step の開始日時を設定する
    pub fn set_started_at(&mut self, started_at: Option<DateTime<Utc>>) {
        self.started_at = started_at;
        self.updated_at = Utc::now();
    }

    /// Step を完了状態にする
    pub fn complete(&mut self, at: DateTime<Utc>) {
        self.completed_at = Some(at);
        self.updated_at = Utc::now();
    }

    /// 指定した ID のパラメーターを削除する
    pub fn remove_parameter(&mut self, parameter_id: &ParameterId) {
        self.parameters.retain(|p| p.id() != parameter_id);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{
        DurationUnit, DurationValue, ParameterContent, ParameterValue,
    };

    fn make_trial_id() -> TrialId {
        TrialId::new()
    }

    #[test]
    fn test_step_id_new_generates_unique_ids() {
        let id1 = StepId::new();
        let id2 = StepId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_step_new_creates_with_correct_initial_state() {
        let trial_id = make_trial_id();
        let step = Step::new(trial_id.clone(), "捏ね".to_string(), 0);

        assert_eq!(step.name(), "捏ね");
        assert_eq!(step.position(), 0);
        assert_eq!(step.trial_id(), &trial_id);
        assert!(step.started_at().is_none());
        assert!(step.completed_at().is_none());
        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_step_start_sets_started_at() {
        let trial_id = make_trial_id();
        let mut step = Step::new(trial_id, "捏ね".to_string(), 0);
        let now = Utc::now();

        step.start(now);

        assert_eq!(step.started_at(), Some(&now));
    }

    #[test]
    fn test_step_add_parameter() {
        let trial_id = make_trial_id();
        let mut step = Step::new(trial_id, "捏ね".to_string(), 0);

        let parameter = Parameter::new(
            step.id().clone(),
            ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            },
        );
        step.add_parameter(parameter);

        assert_eq!(step.parameters().len(), 1);
    }

    #[test]
    fn test_step_add_multiple_parameters() {
        let trial_id = make_trial_id();
        let mut step = Step::new(trial_id, "捏ね".to_string(), 0);

        let param1 = Parameter::new(
            step.id().clone(),
            ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            },
        );
        let param2 = Parameter::new(
            step.id().clone(),
            ParameterContent::Duration {
                duration: DurationValue::new(10.0, DurationUnit::Minute),
                note: "捏ね時間".to_string(),
            },
        );
        step.add_parameter(param1);
        step.add_parameter(param2);

        assert_eq!(step.parameters().len(), 2);
    }
}
