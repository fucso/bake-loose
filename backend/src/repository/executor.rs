//! クエリ実行の共通化

use async_trait::async_trait;
use sqlx::{postgres::PgRow, query::Query, query::QueryAs, FromRow, Postgres};

use crate::ports::RepositoryError;

#[async_trait]
pub trait SqlxQuery<T> {
    async fn execute(&self) -> Result<T, RepositoryError>;
}

macro_rules! impl_sqlx_query {
    ($type:ty, $return:ty, $method:ident) => {
        #[async_trait]
        impl<'q, T> SqlxQuery<$return> for $type
        where
            T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
        {
            async fn execute(&self) -> Result<$return, RepositoryError> {
                self.clone()
                    .$method(&self.pool)
                    .await
                    .map_err(|e| RepositoryError::Internal {
                        message: e.to_string(),
                    })
            }
        }
    };
}

// `QueryAs` のための実装
// impl_sqlx_query!(QueryAs<'q, Postgres, T>, Vec<T>, fetch_all);
// impl_sqlx_query!(QueryAs<'q, Postgres, T>, Option<T>, fetch_optional);
// impl_sqlx_query!(QueryAs<'q, Postgres, T>, T, fetch_one);

// `Query` のための実装
// impl_sqlx_query!(Query<'q, Postgres>, (), execute);
