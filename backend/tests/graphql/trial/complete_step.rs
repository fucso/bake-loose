//! `completeStep` mutation tests

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/trials.sql"))]
async fn test_complete_step(pool: PgPool) {
    let query = r#"
        mutation {
            completeStep(input: {
                trialId: "aaaa0000-0000-0000-0000-000000000001"
                stepId: "bbbb0000-0000-0000-0000-000000000001"
            }) {
                id
                steps {
                    id
                    name
                    completedAt
                }
            }
        }
    "#;

    let data = execute_graphql(pool, query).await;
    let trial = &data["completeStep"];
    let steps = trial["steps"].as_array().unwrap();

    let completed_step = steps
        .iter()
        .find(|s| s["id"] == "bbbb0000-0000-0000-0000-000000000001")
        .unwrap();
    assert!(completed_step["completedAt"].is_string());
}
