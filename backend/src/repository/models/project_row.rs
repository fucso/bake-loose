//! ProjectRow DBモデル

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::project::{Project, ProjectId};
use crate::ports::project_repository::ProjectSortColumn;
use crate::ports::sort::SortColumn;

/// projects テーブルの行を表すDBモデル
#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Project::from_raw(ProjectId(row.id), row.name)
    }
}

/// ProjectSortColumn から DB カラム名へのマッピング
impl SortColumn for ProjectSortColumn {
    fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
