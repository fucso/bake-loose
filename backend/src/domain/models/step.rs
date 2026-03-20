//! Step ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::domain::models::parameter::{Parameter, ParameterContent};
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
    ///
    /// # Arguments
    /// * `at` - 開始時刻。None の場合は現在時刻を使用
    pub fn start(&mut self, at: Option<DateTime<Utc>>) {
        self.started_at = Some(at.unwrap_or_else(Utc::now));
        self.updated_at = Utc::now();
    }

    /// Parameter を追加する
    pub fn add_parameter(&mut self, content: ParameterContent) {
        let parameter = Parameter::new(self.id.clone(), content);
        self.parameters.push(parameter);
        self.updated_at = Utc::now();
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
}
