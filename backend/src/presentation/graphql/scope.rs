//! GraphQL の selection set から `TrialScope` を組み立てるヘルパー
//!
//! read（query）だけでなく write（mutation）でも同じ仕組みを使う。
//! mutation はユースケースが返した集約をそのままレスポンスへ載せるため、
//! 戻り値として要求されたレイヤーを scope に含めないと、DB にデータが
//! 存在していても空配列を返してしまう。

use async_graphql::Lookahead;

use crate::ports::trial_repository::TrialScope;

/// `Trial` を返すフィールドの selection set から `TrialScope` を組み立てる
///
/// `steps { parameters { ... } }` まで要求されていれば `Full`、
/// `steps { ... }` のみ（`parameters` 未選択）なら `WithSteps`、
/// `steps` 自体が選択されていなければ `TrialOnly` とする。
pub fn trial_scope_from_look_ahead(look_ahead: Lookahead<'_>) -> TrialScope {
    let steps = look_ahead.field("steps");
    if !steps.exists() {
        return TrialScope::TrialOnly;
    }

    if steps.field("parameters").exists() {
        TrialScope::Full
    } else {
        TrialScope::WithSteps
    }
}

/// `Step` を返すフィールドの selection set から `TrialScope` を組み立てる
///
/// Step を返す時点で Trial 集約の Step レイヤーは必須のため最低でも `WithSteps`、
/// `parameters` まで要求されていれば `Full` とする。
pub fn step_scope_from_look_ahead(look_ahead: Lookahead<'_>) -> TrialScope {
    if look_ahead.field("parameters").exists() {
        TrialScope::Full
    } else {
        TrialScope::WithSteps
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

    use super::*;

    /// リゾルバーが算出した scope をテスト側へ受け渡すための記録用バッファ
    type RecordedScopes = Arc<Mutex<Vec<TrialScope>>>;

    /// 本番スキーマの `Parameter` を模したダミー型
    #[derive(Default, SimpleObject)]
    #[graphql(name = "Parameter")]
    struct TestParameter {
        id: String,
    }

    /// 本番スキーマの `Step` を模したダミー型
    #[derive(Default, SimpleObject)]
    #[graphql(name = "Step")]
    struct TestStep {
        id: String,
        parameters: Vec<TestParameter>,
    }

    /// 本番スキーマの `Trial` を模したダミー型
    #[derive(Default, SimpleObject)]
    #[graphql(name = "Trial")]
    struct TestTrial {
        id: String,
        steps: Vec<TestStep>,
    }

    struct TestQuery;

    #[Object]
    impl TestQuery {
        /// `Trial` を返すフィールド（query / mutation の Trial 返却に相当）
        async fn trial(&self, ctx: &Context<'_>) -> TestTrial {
            record_scope(ctx, trial_scope_from_look_ahead(ctx.look_ahead()));
            TestTrial::default()
        }

        /// `Step` を返すフィールド（mutation の Step 返却に相当）
        async fn step(&self, ctx: &Context<'_>) -> TestStep {
            record_scope(ctx, step_scope_from_look_ahead(ctx.look_ahead()));
            TestStep::default()
        }
    }

    fn record_scope(ctx: &Context<'_>, scope: TrialScope) {
        ctx.data_unchecked::<RecordedScopes>()
            .lock()
            .unwrap()
            .push(scope);
    }

    /// クエリを実行し、リゾルバーが算出した scope を返す
    async fn scope_of(query: &str) -> TrialScope {
        let recorded: RecordedScopes = Arc::new(Mutex::new(Vec::new()));
        let schema = Schema::build(TestQuery, EmptyMutation, EmptySubscription)
            .data(recorded.clone())
            .finish();

        let response = schema.execute(query).await;
        assert!(
            response.errors.is_empty(),
            "GraphQL errors: {:?}",
            response.errors
        );

        let scopes = recorded.lock().unwrap();
        assert_eq!(scopes.len(), 1, "scope の記録は1件のみを想定: {scopes:?}");
        scopes[0]
    }

    #[tokio::test]
    async fn test_trial_scope_is_trial_only_when_steps_not_selected() {
        assert_eq!(scope_of("{ trial { id } }").await, TrialScope::TrialOnly);
    }

    #[tokio::test]
    async fn test_trial_scope_is_with_steps_when_only_steps_selected() {
        assert_eq!(
            scope_of("{ trial { steps { id } } }").await,
            TrialScope::WithSteps
        );
    }

    #[tokio::test]
    async fn test_trial_scope_is_full_when_parameters_selected() {
        assert_eq!(
            scope_of("{ trial { steps { parameters { id } } } }").await,
            TrialScope::Full
        );
    }

    #[tokio::test]
    async fn test_trial_scope_is_full_when_parameters_selected_via_fragment() {
        // urql / Apollo の codegen が生成する名前付きフラグメント経由の選択でも
        // Lookahead がフラグメントを解決できることを固定する
        let query = r#"query {
            trial { steps { ...StepFields } }
        }
        fragment StepFields on Step {
            id
            parameters { id }
        }"#;

        assert_eq!(scope_of(query).await, TrialScope::Full);
    }

    #[tokio::test]
    async fn test_trial_scope_is_full_when_steps_selected_via_inline_fragment() {
        let query = "{ trial { ... on Trial { steps { parameters { id } } } } }";

        assert_eq!(scope_of(query).await, TrialScope::Full);
    }

    #[tokio::test]
    async fn test_trial_scope_is_full_when_only_one_aliased_steps_selects_parameters() {
        // エイリアスで同名フィールドを重複選択した場合、先に現れる選択だけを見ると
        // WithSteps へ縮退してしまうため、すべての選択がまとめて走査されることを検証する
        let query = "{ trial { a: steps { id } b: steps { parameters { id } } } }";

        assert_eq!(scope_of(query).await, TrialScope::Full);
    }

    #[tokio::test]
    async fn test_step_scope_is_with_steps_when_parameters_not_selected() {
        assert_eq!(scope_of("{ step { id } }").await, TrialScope::WithSteps);
    }

    #[tokio::test]
    async fn test_step_scope_is_full_when_parameters_selected() {
        assert_eq!(
            scope_of("{ step { parameters { id } } }").await,
            TrialScope::Full
        );
    }

    #[tokio::test]
    async fn test_step_scope_is_full_when_parameters_selected_via_fragment() {
        let query = r#"query {
            step { ...StepFields }
        }
        fragment StepFields on Step {
            id
            parameters { id }
        }"#;

        assert_eq!(scope_of(query).await, TrialScope::Full);
    }
}
