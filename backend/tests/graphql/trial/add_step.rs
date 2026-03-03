//! `addStep` mutation tests

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_add_step(pool: PgPool) {
    let query = r#"
        mutation {
            addStep(input: {
                trialId: "aaaa0000-0000-0000-0000-000000000001"
                name: "焼成"
                parameters: [
                    { keyValue: { key: "温度", quantity: { amount: 230.0, unit: "℃" } } }
                ]
            }) {
                id
                steps {
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
                        }
                    }
                }
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["addStep"];
    let steps = trial["steps"].as_array().unwrap();

    // 既存の2ステップ + 新規1ステップ = 3
    assert_eq!(steps.len(), 3);

    let new_step = &steps[2];
    assert_eq!(new_step["name"], "焼成");
    assert_eq!(new_step["position"], 2);

    let params = new_step["parameters"].as_array().unwrap();
    assert_eq!(params.len(), 1);
}
