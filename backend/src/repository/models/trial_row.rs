//! TrialRow DBモデル

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::project::ProjectId;
use crate::domain::models::step::Step;
use crate::domain::models::trial::{Trial, TrialId, TrialStatus};
use crate::domain::timezone::JstDateTime;
use crate::ports::error::RepositoryError;

/// trials テーブルの行を表すDBモデル
#[derive(Debug, FromRow)]
pub struct TrialRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TrialRow {
    /// Step一覧と合わせてドメインモデルに変換する
    ///
    /// `status` が未知の値の場合はデータ破損の兆候のため、
    /// 無言で `InProgress` 扱いにせずエラーとして返す。
    pub fn into_domain(self, steps: Vec<Step>) -> Result<Trial, RepositoryError> {
        let status = match self.status.as_str() {
            "in_progress" => TrialStatus::InProgress,
            "completed" => TrialStatus::Completed,
            other => {
                return Err(RepositoryError::Internal {
                    message: format!("unknown trial status: {other}"),
                })
            }
        };

        Ok(Trial::from_raw(
            TrialId(self.id),
            ProjectId(self.project_id),
            self.name,
            self.memo,
            status,
            self.completed_at.map(JstDateTime::from_utc),
            steps,
        ))
    }

    /// TrialStatus を DB カラム値へ変換する
    pub fn status_column(status: &TrialStatus) -> &'static str {
        match status {
            TrialStatus::InProgress => "in_progress",
            TrialStatus::Completed => "completed",
        }
    }
}
