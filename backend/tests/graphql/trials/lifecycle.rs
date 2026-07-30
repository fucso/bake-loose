//! Trial 作成〜Step追加〜完了までの一連の GraphQL 操作を通しで検証する

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

const PROJECT_ID: &str = "11111111-1111-1111-1111-111111111111";

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_full_trial_lifecycle(pool: PgPool) {
    // 1. Trialを作成する
    let create_query = format!(
        r#"mutation {{
            createTrial(input: {{ projectId: "{PROJECT_ID}", name: "焼成温度検証" }}) {{ id status }}
        }}"#
    );
    let created = execute_graphql(pool.clone(), &create_query).await;
    let trial_id = created["createTrial"]["id"].as_str().unwrap().to_string();
    assert_eq!(created["createTrial"]["status"], "IN_PROGRESS");

    // 2. Stepを追加する
    let add_step_query = format!(
        r#"mutation {{
            addStep(trialId: "{trial_id}", input: {{ name: "こね" }}) {{ id name isCompleted }}
        }}"#
    );
    let added = execute_graphql(pool.clone(), &add_step_query).await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();
    assert_eq!(added["addStep"]["name"], "こね");
    assert_eq!(added["addStep"]["isCompleted"], false);

    // 3. Stepを更新する
    let update_step_query = format!(
        r#"mutation {{
            updateStep(trialId: "{trial_id}", stepId: "{step_id}", input: {{ name: "一次発酵" }}) {{
                name
            }}
        }}"#
    );
    let updated = execute_graphql(pool.clone(), &update_step_query).await;
    assert_eq!(updated["updateStep"]["name"], "一次発酵");

    // 4. Stepを完了する
    let complete_step_query = format!(
        r#"mutation {{
            completeStep(trialId: "{trial_id}", stepId: "{step_id}") {{ isCompleted }}
        }}"#
    );
    let step_completed = execute_graphql(pool.clone(), &complete_step_query).await;
    assert_eq!(step_completed["completeStep"]["isCompleted"], true);

    // 5. Trialを完了する
    let complete_trial_query =
        format!(r#"mutation {{ completeTrial(id: "{trial_id}") {{ status }} }}"#);
    let trial_completed = execute_graphql(pool.clone(), &complete_trial_query).await;
    assert_eq!(trial_completed["completeTrial"]["status"], "COMPLETED");

    // 6. trial クエリで最終状態を確認する
    let get_query = format!(
        r#"{{
            trial(id: "{trial_id}") {{
                status
                steps {{ name isCompleted }}
            }}
        }}"#
    );
    let final_state = execute_graphql(pool.clone(), &get_query).await;
    assert_eq!(final_state["trial"]["status"], "COMPLETED");
    assert_eq!(
        final_state["trial"]["steps"],
        serde_json::json!([{ "name": "一次発酵", "isCompleted": true }])
    );

    // 7. trialsByProject クエリでも取得できることを確認する
    let list_query = format!(r#"{{ trialsByProject(projectId: "{PROJECT_ID}") {{ id status }} }}"#);
    let list = execute_graphql(pool, &list_query).await;
    assert_eq!(
        list["trialsByProject"],
        serde_json::json!([{ "id": trial_id, "status": "COMPLETED" }])
    );
}
