//! `trial` クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations")]
async fn test_returns_null_when_not_found(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trial(id: "00000000-0000-0000-0000-000000000000") { id name } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trial": null }));
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_trial(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                projectId
                name
                memo
                status
                steps { id }
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "projectId": "11111111-1111-1111-1111-111111111111",
                "name": "Test Trial 1",
                "memo": "Test Memo",
                "status": "IN_PROGRESS",
                "steps": []
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_returns_trial_without_steps_when_steps_not_requested(pool: PgPool) {
    // Step・Parameter が存在していても、問い合わせていないフィールドは返らないことを検証する
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                projectId
                name
                memo
                status
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "projectId": "11111111-1111-1111-1111-111111111111",
                "name": "Test Trial 1",
                "memo": "Test Memo",
                "status": "IN_PROGRESS"
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_returns_trial_with_steps_but_without_parameters_when_parameters_not_requested(
    pool: PgPool,
) {
    // Parameter が存在していても、steps 配下で問い合わせていない parameters は
    // 返らないことを検証する（WithSteps スコープ）
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                steps {
                    id
                    name
                    position
                    isCompleted
                }
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "steps": [
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "name": "こね",
                        "position": 0,
                        "isCompleted": true
                    },
                    {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "name": "一次発酵",
                        "position": 1,
                        "isCompleted": false
                    }
                ]
            }
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures(
        "../../fixtures/projects.sql",
        "../../fixtures/trials.sql",
        "../../fixtures/steps.sql"
    )
)]
async fn test_returns_trial_with_steps_and_parameters(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                projectId
                name
                memo
                status
                steps {
                    id
                    name
                    position
                    startedAt
                    completedAt
                    isCompleted
                    parameters { id parameterType content }
                }
            }
        }"#,
    )
    .await;

    // Step は position 昇順、Parameter はフィクスチャの投入順で返るため完全なJSONで検証する
    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "projectId": "11111111-1111-1111-1111-111111111111",
                "name": "Test Trial 1",
                "memo": "Test Memo",
                "status": "IN_PROGRESS",
                "steps": [
                    {
                        "id": "77777777-7777-7777-7777-777777777777",
                        "name": "こね",
                        "position": 0,
                        "startedAt": "2026-01-01T09:00:00+09:00",
                        "completedAt": "2026-01-01T09:30:00+09:00",
                        "isCompleted": true,
                        "parameters": [
                            {
                                "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1",
                                "parameterType": "TEXT",
                                "content": { "type": "text", "value": "打ち粉を追加" }
                            },
                            {
                                "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2",
                                "parameterType": "KEY_VALUE",
                                "content": {
                                    "type": "key_value",
                                    "key": "強力粉",
                                    "value": { "type": "quantity", "amount": 300.0, "unit": "g" }
                                }
                            }
                        ]
                    },
                    {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "name": "一次発酵",
                        "position": 1,
                        "startedAt": null,
                        "completedAt": null,
                        "isCompleted": false,
                        "parameters": [
                            {
                                "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1",
                                "parameterType": "DURATION",
                                "content": {
                                    "type": "duration",
                                    "duration": { "value": 90.0, "unit": "minute" },
                                    "note": "一次発酵"
                                }
                            },
                            {
                                "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2",
                                "parameterType": "TIME_MARKER",
                                "content": {
                                    "type": "time_marker",
                                    "at": { "value": 60.0, "unit": "minute" },
                                    "note": "生地の膨らみを確認"
                                }
                            }
                        ]
                    }
                ]
            }
        })
    );
}
