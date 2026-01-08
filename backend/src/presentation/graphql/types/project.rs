//! Project GraphQL 型
//!
//! ドメインモデルの Project をラップした GraphQL 型。

use async_graphql::{InputObject, Object, ID};

use crate::domain::models::project::Project as DomainProject;

/// GraphQL 用の Project 型
///
/// ドメインモデルを直接公開せず、ラッパー型として定義する。
pub struct Project(pub DomainProject);

#[Object]
impl Project {
    /// プロジェクトID
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    /// プロジェクト名
    async fn name(&self) -> &str {
        self.0.name()
    }
}

impl From<DomainProject> for Project {
    fn from(project: DomainProject) -> Self {
        Self(project)
    }
}

/// プロジェクト作成時の入力
#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
}
