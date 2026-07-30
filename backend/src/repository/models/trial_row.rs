//! TrialRow DBモデル

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;
use crate::domain::models::trial::{Trial, TrialId, TrialStatus};

/// trials テーブルの行を表すDBモデル
#[derive(Debug, FromRow)]
pub struct TrialRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrialRow {
    /// Step一覧と合わせてドメインモデルに変換する
    pub fn into_domain(self, steps: Vec<Step>) -> Trial {
        let status = match self.status.as_str() {
            "completed" => TrialStatus::Completed,
            _ => TrialStatus::InProgress,
        };

        Trial::from_raw(
            TrialId(self.id),
            ProjectId(self.project_id),
            self.name,
            self.memo,
            status,
            steps,
        )
    }

    /// TrialStatus を DB カラム値へ変換する
    pub fn status_column(status: &TrialStatus) -> &'static str {
        match status {
            TrialStatus::InProgress => "in_progress",
            TrialStatus::Completed => "completed",
        }
    }
}
