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

/** JST の UTC オフセット。バックエンドが日時を JST で扱うため、送信時もこれを付与する */
const JST_OFFSET = "+09:00"

/**
 * `<input type="datetime-local">` の値を組み立てるためのフォーマッタ。
 *
 * 入力欄は閲覧端末のタイムゾーンを持たない「壁時計の時刻」を扱うため、
 * 表示と同じく JST の壁時計時刻に変換してから流し込む。
 * `hourCycle: "h23"` は 0 時を `24` と表記する実装差を避けるために指定する。
 */
const DATE_TIME_LOCAL_FORMATTER = new Intl.DateTimeFormat("en-CA", {
  timeZone: "Asia/Tokyo",
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  hourCycle: "h23",
})

/** `<input type="datetime-local">` が受け付ける形式（step 指定により秒・ミリ秒が付くことがある） */
const DATE_TIME_LOCAL_PATTERN = /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2})(:\d{2}(?:\.\d{1,3})?)?$/

/**
 * ISO 8601 文字列を `<input type="datetime-local">` の値（`2026-01-01T09:00`）に変換する。
 *
 * 未設定・解釈できない文字列は空欄（未入力）として扱う。
 */
const toDateTimeLocalValue = (isoString: string | null): string => {
  if (isoString === null) {
    return ""
  }

  const date = new Date(isoString)
  if (Number.isNaN(date.getTime())) {
    return ""
  }

  const parts = new Map(
    DATE_TIME_LOCAL_FORMATTER.formatToParts(date).map((part) => [part.type, part.value])
  )
  const [year, month, day, hour, minute] = (["year", "month", "day", "hour", "minute"] as const).map(
    (type) => parts.get(type)
  )
  // 指定した書式なら全パートが揃うが、欠けた値をそのまま入力欄へ流し込まないよう空欄にする
  if (!year || !month || !day || !hour || !minute) {
    return ""
  }

  return `${year}-${month}-${day}T${hour}:${minute}`
}

/**
 * `<input type="datetime-local">` の値を JST オフセット付き ISO 8601 文字列に変換する。
 *
 * 入力欄の値はタイムゾーンを持たないため、JST の壁時計時刻として解釈する。
 * 空欄・未完成の入力は「未設定」を意味する null を返す。
 */
const fromDateTimeLocalValue = (value: string): string | null => {
  const matched = DATE_TIME_LOCAL_PATTERN.exec(value.trim())
  if (matched === null) {
    return null
  }
  return `${matched[1]}${matched[2] ?? ":00"}${JST_OFFSET}`
}

export { formatDateTime, fromDateTimeLocalValue, toDateTimeLocalValue }
