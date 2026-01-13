//! Step ドメインモデル

use super::Parameter;
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    id: StepId,
    name: Option<String>,
    position: u8,
    started_at: DateTime<Utc>,
    parameters: Vec<Parameter>,
}

impl Step {
    /// 新しい Step を作成する（Parameters は空）
    pub fn new(position: u8, started_at: DateTime<Utc>) -> Self {
        Self {
            id: StepId::new(),
            name: None,
            position,
            started_at,
            parameters: Vec::new(),
        }
    }

    /// DB などから Step を復元する
    pub fn from_raw(
        id: StepId,
        name: Option<String>,
        position: u8,
        started_at: DateTime<Utc>,
        parameters: Vec<Parameter>,
    ) -> Self {
        Self {
            id,
            name,
            position,
            started_at,
            parameters,
        }
    }

    // Getters
    pub fn id(&self) -> &StepId {
        &self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn position(&self) -> u8 {
        self.position
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Parameter を追加する
    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(parameter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::trial::{ParameterContent, TextParameter};

    #[test]
    fn test_step_new_has_empty_parameters() {
        let started_at = Utc::now();
        let step = Step::new(0, started_at);
        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_step_new_has_started_at() {
        let started_at = Utc::now();
        let step = Step::new(0, started_at);
        assert_eq!(step.started_at(), started_at);
    }

    #[test]
    fn test_step_add_parameter() {
        let started_at = Utc::now();
        let mut step = Step::new(0, started_at);
        let param = Parameter::new(ParameterContent::Text(TextParameter {
            value: "Test".to_string(),
        }));
        step.add_parameter(param);
        assert_eq!(step.parameters().len(), 1);
    }
}
