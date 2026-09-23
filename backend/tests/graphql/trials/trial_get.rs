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
    // parameters を選択せず steps のみを問い合わせた場合（WithSteps スコープ）でも、
    // steps が欠落せずに返ることを検証する。
    // parameters テーブルへ SELECT が飛んでいないこと自体は GraphQL レスポンスからは
    // 観測できないため、その検証は repository 層の
    // `test_find_by_id_with_with_steps_scope_fetches_steps_without_parameters` が担う
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
async fn test_returns_parameters_when_selected_via_fragment(pool: PgPool) {
    // 名前付きフラグメント経由で parameters を選択した場合でも scope が Full と判定され、
    // parameters が空配列に縮退しないことを検証する。
    // urql / Apollo の codegen はフラグメントを展開せずそのまま送るため、
    // インラインで直接選択するケースと同等に扱えている必要がある
    let data = execute_graphql(
        pool,
        r#"query {
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                steps { ...StepFields }
            }
        }
        fragment StepFields on Step {
            id
            parameters { id }
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
                        "parameters": [
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1" },
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2" }
                        ]
                    },
                    {
                        "id": "88888888-8888-8888-8888-888888888888",
                        "parameters": [
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1" },
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2" }
                        ]
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
async fn test_returns_parameters_when_only_one_aliased_steps_requests_them(pool: PgPool) {
    // 同名フィールドをエイリアスで重複選択し、片方だけが parameters を要求するケース。
    // 先に現れる `a: steps` だけを見て scope を決めてしまうと parameters が
    // 空配列へ縮退するため、すべての選択をまとめて走査できていることを検証する
    let data = execute_graphql(
        pool,
        r#"{
            trial(id: "33333333-3333-3333-3333-333333333333") {
                id
                a: steps { id }
                b: steps { parameters { id } }
            }
        }"#,
    )
    .await;

    assert_eq!(
        data,
        json!({
            "trial": {
                "id": "33333333-3333-3333-3333-333333333333",
                "a": [
                    { "id": "77777777-7777-7777-7777-777777777777" },
                    { "id": "88888888-8888-8888-8888-888888888888" }
                ],
                "b": [
                    {
                        "parameters": [
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa1" },
                            { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaa2" }
                        ]
                    },
                    {
                        "parameters": [
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb1" },
                            { "id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbb2" }
                        ]
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
