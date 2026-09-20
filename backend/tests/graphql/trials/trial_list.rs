//! `trialsByProject` クエリのテスト

use serde_json::json;
use sqlx::PgPool;

use crate::graphql::schema::execute_graphql;

#[sqlx::test(migrations = "./migrations", fixtures("../../fixtures/projects.sql"))]
async fn test_returns_empty_list_when_no_trials(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") { id } }"#,
    )
    .await;

    assert_eq!(data, json!({ "trialsByProject": [] }));
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_only_trials_for_specified_project(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "22222222-2222-2222-2222-222222222222") { id name } }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "55555555-5555-5555-5555-555555555555",
                    "name": "Other Project Trial"
                }
            ]
        })
    );
}

#[sqlx::test(
    migrations = "./migrations",
    fixtures("../../fixtures/projects.sql", "../../fixtures/trials.sql")
)]
async fn test_returns_all_trials_for_project_with_multiple_trials(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{ trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") { id name } }"#,
    )
    .await;

    // trial_repo.rs の find_all_by_project は created_at, id 順に決定的にソートするため、
    // フィクスチャの投入順（id昇順）と一致する完全なJSONで検証する
    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "name": "Test Trial 1"
                },
                {
                    "id": "44444444-4444-4444-4444-444444444444",
                    "name": "Test Trial 2"
                },
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "name": "Completed Trial"
                }
            ]
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
async fn test_returns_trials_without_steps_when_steps_not_requested(pool: PgPool) {
    // Step・Parameter が存在していても、問い合わせていないフィールドは返らないことを検証する
    let data = execute_graphql(
        pool,
        r#"{
            trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") {
                id
                name
                status
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "name": "Test Trial 1",
                    "status": "IN_PROGRESS"
                },
                {
                    "id": "44444444-4444-4444-4444-444444444444",
                    "name": "Test Trial 2",
                    "status": "IN_PROGRESS"
                },
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "name": "Completed Trial",
                    "status": "COMPLETED"
                }
            ]
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
async fn test_returns_trials_with_steps_and_parameters(pool: PgPool) {
    let data = execute_graphql(
        pool,
        r#"{
            trialsByProject(projectId: "11111111-1111-1111-1111-111111111111") {
                id
                name
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

    // Trial は created_at, id 順、Step は position 昇順、
    // Parameter はフィクスチャの投入順で返るため完全なJSONで検証する
    assert_eq!(
        data,
        json!({
            "trialsByProject": [
                {
                    "id": "33333333-3333-3333-3333-333333333333",
                    "name": "Test Trial 1",
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
                },
                {
                    "id": "44444444-4444-4444-4444-444444444444",
                    "name": "Test Trial 2",
                    "steps": []
                },
                {
                    "id": "66666666-6666-6666-6666-666666666666",
                    "name": "Completed Trial",
                    "steps": []
                }
            ]
        })
    );
}
