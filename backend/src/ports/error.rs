//! リポジトリ層のエラー型

/// リポジトリ操作で発生するエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// データが見つからない
    NotFound { entity: String, id: String },
    /// 一意性制約違反
    ///
    /// `field` には違反したカラム名相当の値が入る（例: `name` / `trial_id_position` / `id`）。
    /// 呼び出し側が文字列比較で違反箇所を絞り込めるよう、制約名そのものは入れない。
    Conflict { entity: String, field: String },
    /// 削除・更新しようとした行が他の行から参照されている
    ///
    /// `constraint` には違反した外部キー制約名がそのまま入る（例: `trials_project_id_fkey`）。
    /// リトライでは解消しないため `Conflict` とは区別する。
    StillReferenced { entity: String, constraint: String },
    /// 接続エラー
    Connection,
    /// その他の内部エラー
    Internal { message: String },
}
