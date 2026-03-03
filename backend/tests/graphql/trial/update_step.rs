//! `updateStep` mutation tests

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_update_step(pool: PgPool) {
    let query = r#"
        mutation {
            updateStep(input: {
                trialId: "aaaa0000-0000-0000-0000-000000000001"
                stepId: "bbbb0000-0000-0000-0000-000000000001"
                name: "捏ね（更新）"
            }) {
                id
                steps {
                    id
                    name
                }
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["updateStep"];
    let steps = trial["steps"].as_array().unwrap();

    let updated_step = steps
        .iter()
        .find(|s| s["id"] == "bbbb0000-0000-0000-0000-000000000001")
        .unwrap();
    assert_eq!(updated_step["name"], "捏ね（更新）");
}
