//! Project ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// プロジェクトID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    /// 新しいプロジェクトIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// プロジェクト（調理テーマ）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    id: ProjectId,
    name: String,
}

impl Project {
    /// 新しいプロジェクトを作成する（ID は自動生成）
    pub fn new(name: String) -> Self {
        Self {
            id: ProjectId::new(),
            name,
        }
    }

    /// 生データからプロジェクトを構築する
    pub fn from_raw(id: ProjectId, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_new_generates_unique_ids() {
        let id1 = ProjectId::new();
        let id2 = ProjectId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_project_new_creates_with_auto_id() {
        let project = Project::new("ピザ生地研究".to_string());
        assert_eq!(project.name(), "ピザ生地研究");
    }
}
