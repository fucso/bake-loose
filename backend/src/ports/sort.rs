//! ソート関連の汎用型
//!
//! モデルに依存しない、SQL の並び順を表現するリソース。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// 各モデルの SortColumn enum がこのトレイトを実装する。
pub trait SortColumn: Send + Sync + Copy {
    fn as_sql_column(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort<C: SortColumn> {
    pub column: C,
    pub direction: SortDirection,
}

impl<C: SortColumn> Sort<C> {
    pub fn new(column: C, direction: SortDirection) -> Self {
        Self { column, direction }
    }

    pub fn asc(column: C) -> Self {
        Self::new(column, SortDirection::Asc)
    }

    pub fn desc(column: C) -> Self {
        Self::new(column, SortDirection::Desc)
    }

    /// 例: "ORDER BY name ASC", "ORDER BY created_at DESC"
    pub fn to_order_by_clause(&self) -> String {
        format!(
            "ORDER BY {} {}",
            self.column.as_sql_column(),
            self.direction.as_sql()
        )
    }
}

impl<C: SortColumn + Default> Default for Sort<C> {
    fn default() -> Self {
        Self {
            column: C::default(),
            direction: SortDirection::default(),
        }
    }
}
