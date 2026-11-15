/**
 * 日時の表示整形ユーティリティ
 *
 * バックエンドは日時を JST（+09:00）のオフセット付き ISO 8601 文字列で返す。
 * 閲覧端末のタイムゾーン設定によって表示がぶれないよう、整形は Asia/Tokyo 固定で行う。
 */

const DATE_TIME_FORMATTER = new Intl.DateTimeFormat("ja-JP", {
  timeZone: "Asia/Tokyo",
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
})

/**
 * ISO 8601 日時文字列を `2026/03/01 09:30` 形式（JST）に整形する。
 *
 * @param value ISO 8601 形式の日時文字列
 * @returns 整形済みの文字列。パースできない値の場合は `null`
 */
export const formatDateTime = (value: string): string | null => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return null
  }
  return DATE_TIME_FORMATTER.format(date)
}
