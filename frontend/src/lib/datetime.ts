/**
 * バックエンドが返す日時（JST オフセット付き ISO 8601 文字列）を表示用に整形する。
 *
 * バックエンドは日時を JST（`DateTime<FixedOffset>`）で返すため、閲覧端末の
 * タイムゾーン設定に関わらず JST で表示する。
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
 * ISO 8601 文字列を `2026/01/01 09:00` 形式に整形する。
 * 解釈できない文字列は情報を失わないよう元の文字列をそのまま返す。
 */
const formatDateTime = (isoString: string): string => {
  const date = new Date(isoString)
  if (Number.isNaN(date.getTime())) {
    return isoString
  }
  return DATE_TIME_FORMATTER.format(date)
}

export { formatDateTime }
