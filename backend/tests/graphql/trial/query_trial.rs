//! trial クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_null_when_not_found(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trial(id: "00000000-0000-0000-0000-000000000000") { id } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trial": null }));
}

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_query_trial(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "aaaa0000-0000-0000-0000-000000000001") {
                id
                projectId
                name
                memo
                status
                steps {
                    id
                    name
                    position
                    parameters {
                        id
                        content {
                            ... on KeyValueParameter {
                                key
                                value {
                                    ... on QuantityValue {
                                        amount
                                        unit
                                    }
                                }
                            }
                            ... on TextParameter {
                                value
                            }
                        }
                    }
                }
            }
        }"#,
    )
    .await;

    let trial = &data["trial"];
    assert_eq!(trial["id"], "aaaa0000-0000-0000-0000-000000000001");
    assert_eq!(trial["projectId"], "11111111-1111-1111-1111-111111111111");
    assert_eq!(trial["name"], "Trial 1");
    assert_eq!(trial["memo"], "メモ1");
    assert_eq!(trial["status"], "IN_PROGRESS");
    assert_eq!(trial["steps"].as_array().unwrap().len(), 2);

    let step0 = &trial["steps"][0];
    assert_eq!(step0["name"], "捏ね");
    assert_eq!(step0["position"], 0);
    assert_eq!(step0["parameters"].as_array().unwrap().len(), 2);
}
