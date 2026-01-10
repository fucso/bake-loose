//! Step ドメインモデル

use super::trial::TrialId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Step の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}


/// ステップ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    trial_id: TrialId,
    name: Option<String>,
    position: u8,
    started_at: Option<DateTime<Utc>>,
}

impl Step {
    /// 新しい Step を作成する
    pub fn new(trial_id: TrialId, position: u8) -> Self {
        Self {
            id: StepId::new(),
            trial_id,
            name: None,
            position,
            started_at: None,
        }
    }

    /// DB などから Step を復元する
    pub fn from_raw(
        id: StepId,
        trial_id: TrialId,
        name: Option<String>,
        position: u8,
        started_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            trial_id,
            name,
            position,
            started_at,
        }
    }

    // Getters
    pub fn id(&self) -> &StepId {
        &self.id
    }

    pub fn trial_id(&self) -> &TrialId {
        &self.trial_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn position(&self) -> u8 {
        self.position
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_new_creates_with_given_position() {
        let trial_id = TrialId::new();
        let step = Step::new(trial_id.clone(), 3);
        assert_eq!(step.trial_id(), &trial_id);
        assert_eq!(step.position(), 3);
        assert!(step.name().is_none());
        assert!(step.started_at().is_none());
    }
}
