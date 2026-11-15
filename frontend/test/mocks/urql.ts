import { CombinedError, Client, type Exchange } from 'urql'
import { Kind, type DocumentNode, type OperationDefinitionNode } from 'graphql'
import { map, pipe, tap } from 'wonka'

/**
 * `MockGraphQLError` をレスポンスとして登録すると、対応するクエリ/ミューテーションは
 * データの代わりに GraphQL エラー（`CombinedError`）を返す。エラー状態のテストに使用する。
 *
 * @example
 * const client = createMockClient({ Projects: new MockGraphQLError('failed') })
 */
export class MockGraphQLError {
  constructor(public message: string) {}
}

/**
 * GraphQLクエリ/ミューテーション名をキーに、返却したいレスポンスデータをマッピングしたもの。
 * 例: { GraphqlSmokeTest: { __typename: 'Query' } }
 * エラーレスポンスを返したい場合は値に `MockGraphQLError` を指定する。
 */
export type MockQueryResponses = Record<string, unknown | MockGraphQLError>

const getOperationName = (query: DocumentNode): string | undefined => {
  const definition = query.definitions.find(
    (def): def is OperationDefinitionNode => def.kind === Kind.OPERATION_DEFINITION,
  )
  return definition?.name?.value
}

/**
 * `responses` に登録したクエリ/ミューテーション名に対して、実際のネットワーク通信を行わずに
 * 即座にレスポンスを返す urql の Exchange を作成する。
 * 未登録のクエリ名の場合は `data: undefined` を返す（未モックであることが分かるようにするため）。
 */
export const createMockExchange = (responses: MockQueryResponses): Exchange => {
  return () => (ops$) =>
    pipe(
      ops$,
      map((operation) => {
        const operationName = getOperationName(operation.query)
        const response = operationName ? responses[operationName] : undefined
        const isMockError = response instanceof MockGraphQLError

        return {
          operation,
          data: isMockError ? undefined : response,
          error: isMockError
            ? new CombinedError({ graphQLErrors: [response.message] })
            : undefined,
          extensions: undefined,
          hasNext: false,
          stale: false,
        }
      }),
    )
}

/**
 * `responses` をモックした GraphQL レスポンスとして返す urql Client を生成する。
 * テストでは `urql` の `Provider` にこの Client を渡してコンポーネントをレンダリングする。
 *
 * @example
 * const client = createMockClient({ SomeQuery: { __typename: 'Query' } })
 * render(
 *   <Provider value={client}>
 *     <YourComponent />
 *   </Provider>,
 * )
 */
export const createMockClient = (responses: MockQueryResponses): Client => {
  return new Client({
    url: 'http://mock.test/graphql',
    exchanges: [createMockExchange(responses)],
  })
}

/** 記録された1件の GraphQL 操作 */
export type RecordedOperation = {
  /** クエリ/ミューテーション名 */
  name: string
  variables: Record<string, unknown>
}

/**
 * `createMockClient` と同じレスポンスを返しつつ、送信された操作名と変数を記録する Client を生成する。
 *
 * 画面の入力内容がミューテーションの変数へ正しく組み立てられているかを検証したいときに使う。
 * `operations` は呼び出し順に追記される。
 *
 * @example
 * const { client, operations } = createRecordingClient({ AddParameter: { addParameter: { id: '1' } } })
 * // ...操作後
 * expect(operations[0].variables).toEqual({ ... })
 */
export const createRecordingClient = (
  responses: MockQueryResponses,
): { client: Client; operations: RecordedOperation[] } => {
  const operations: RecordedOperation[] = []

  const respond = createMockExchange(responses)

  const recordingExchange: Exchange = (input) => (ops$) =>
    respond(input)(
      pipe(
        ops$,
        tap((operation) => {
          operations.push({
            name: getOperationName(operation.query) ?? '',
            variables: (operation.variables ?? {}) as Record<string, unknown>,
          })
        }),
      ),
    )

  return {
    client: new Client({ url: 'http://mock.test/graphql', exchanges: [recordingExchange] }),
    operations,
  }
}
