//! Trial ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;

/// トライアルID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrialId(pub Uuid);

impl TrialId {
    /// 新しいトライアルIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrialId {
    fn default() -> Self {
        Self::new()
    }
}

/// トライアルのステータス
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialStatus {
    InProgress,
    Completed,
}

/// トライアル（プロジェクトに紐づく試行記録）
///
/// aggregate root として Steps を含む。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trial {
    id: TrialId,
    project_id: ProjectId,
    name: Option<String>,
    memo: Option<String>,
    status: TrialStatus,
    steps: Vec<Step>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Trial {
    /// 新しいトライアルを作成する（ID は自動生成、ステータスは InProgress）
    pub fn new(project_id: ProjectId, name: Option<String>, memo: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: TrialId::new(),
            project_id,
            name,
            memo,
            status: TrialStatus::InProgress,
            steps: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 生データからトライアルを構築する
    #[allow(clippy::too_many_arguments)]
    pub fn from_raw(
        id: TrialId,
        project_id: ProjectId,
        name: Option<String>,
        memo: Option<String>,
        status: TrialStatus,
        steps: Vec<Step>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            memo,
            status,
            steps,
            created_at,
            updated_at,
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

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Steps への可変参照を返す
    pub fn steps_mut(&mut self) -> &mut Vec<Step> {
        &mut self.steps
    }

    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.updated_at
    }

    /// Step を追加する
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
        self.updated_at = chrono::Utc::now();
    }

    /// 次の Step の position を返す
    pub fn next_step_position(&self) -> i16 {
        self.steps.len() as i16
    }

    /// Trial の名前を設定する
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
        self.updated_at = chrono::Utc::now();
    }

    /// Trial のメモを設定する
    pub fn set_memo(&mut self, memo: Option<String>) {
        self.memo = memo;
        self.updated_at = chrono::Utc::now();
    }

    /// Trial を完了状態にする
    pub fn complete(&mut self) {
        self.status = TrialStatus::Completed;
        self.updated_at = chrono::Utc::now();
    }

    /// updated_at を現在時刻に更新する
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
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
    fn test_trial_new_creates_with_in_progress_status() {
        let project_id = ProjectId::new();
        let trial = Trial::new(project_id, Some("バゲット第1回".to_string()), None);
        assert_eq!(trial.status(), &TrialStatus::InProgress);
        assert_eq!(trial.name(), Some("バゲット第1回"));
        assert!(trial.steps().is_empty());
    }
}
