//! Step / Parameter 系 mutation テストの共通ヘルパー
//!
//! 各 mutation ごとに分割されたテストファイルから共通で参照する。

use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

pub const TRIAL_ID: &str = "33333333-3333-3333-3333-333333333333";
pub const COMPLETED_TRIAL_ID: &str = "66666666-6666-6666-6666-666666666666";

/// `addStep` で Step を追加し、そのレスポンスを返す
pub async fn add_step(pool: PgPool, trial_id: &str, name: &str) -> serde_json::Value {
    let query = format!(
        r#"
        mutation {{
            addStep(trialId: "{trial_id}", input: {{
                name: "{name}"
            }}) {{
                id
                name
                position
                isCompleted
                parameters {{ id content }}
            }}
        }}
        "#
    );
    execute_graphql(pool, &query).await
}

/// `addParameter` で Step にパラメーターを1件追加し、そのレスポンスを返す
pub async fn add_parameter(
    pool: PgPool,
    trial_id: &str,
    step_id: &str,
    content_literal: &str,
) -> serde_json::Value {
    let query = format!(
        r#"
        mutation {{
            addParameter(trialId: "{trial_id}", stepId: "{step_id}", content: {content_literal}) {{
                id
                parameterType
                content
            }}
        }}
        "#
    );
    execute_graphql(pool, &query).await
}

/// `addStep` で Step を作成し、`addParameter` でパラメーターを2件（text, key_value）付与する
///
/// 付与後の状態は `trial` クエリで取得し、追加した Step（`addStep` が返した id）の
/// JSON をそのまま返す。
pub async fn add_step_with_parameters(
    pool: PgPool,
    trial_id: &str,
    name: &str,
) -> serde_json::Value {
    let added = add_step(pool.clone(), trial_id, name).await;
    let step_id = added["addStep"]["id"].as_str().unwrap().to_string();

    add_parameter(
        pool.clone(),
        trial_id,
        &step_id,
        r#"{ type: "text", value: "打ち粉を追加" }"#,
    )
    .await;
    add_parameter(
        pool.clone(),
        trial_id,
        &step_id,
        r#"{ type: "key_value", key: "強力粉", value: { type: "quantity", amount: 300, unit: "g" } }"#,
    )
    .await;

    let query = format!(
        r#"
        {{
            trial(id: "{trial_id}") {{
                steps {{
                    id
                    name
                    position
                    isCompleted
                    parameters {{ id parameterType content }}
                }}
            }}
        }}
        "#
    );
    let data = execute_graphql(pool, &query).await;

    data["trial"]["steps"]
        .as_array()
        .expect("trial の steps が取得できること")
        .iter()
        .find(|step| step["id"] == step_id.as_str())
        .expect("追加した Step が取得できること")
        .clone()
}
