import type { CombinedError } from "urql"

/**
 * urql のエラーからユーザー向けのメッセージを取り出す。
 *
 * バックエンドの GraphQL エラーメッセージ（名称の重複・文字数超過など）は
 * presentation 層（`backend/src/presentation/graphql/error.rs`）で既にユーザー向けに
 * 変換済みのため、そのまま表示する。GraphQL エラーが無い場合（ネットワークエラー等）は
 * 呼び出し側が渡した汎用メッセージにフォールバックする。
 *
 * @param error urql のミューテーション/クエリ結果のエラー
 * @param fallbackMessage GraphQL エラーが含まれない場合に表示するメッセージ
 * @returns 表示するエラーメッセージ。エラーが無い場合は null
 */
const toUserFacingErrorMessage = (
  error: CombinedError | undefined,
  fallbackMessage: string
): string | null => {
  if (!error) {
    return null
  }
  return error.graphQLErrors[0]?.message ?? fallbackMessage
}

export { toUserFacingErrorMessage }
