//! リポジトリ層のエラー型

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    NotFound {
        entity: String,
        id: String,
    },
    /// `field` には違反したカラム名相当の値が入る（例: `name` / `trial_id_position` / `id`）。
    Conflict {
        entity: String,
        field: String,
    },
    /// 削除・更新しようとした行が他の行から参照されている
    /// `constraint` には違反した外部キー制約名がそのまま入る（例: `trials_project_id_fkey`）。
    StillReferenced {
        entity: String,
        constraint: String,
    },
    Connection,
    Internal {
        message: String,
    },
}
