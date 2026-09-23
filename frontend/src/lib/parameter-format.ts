import type { DurationUnit, DurationValue, Parameter, ParameterValue } from "@/lib/trial"

/** 表示用に整形した Parameter */
type FormattedParameter = {
  /** 値の意味を示すラベル。自由記述のようにラベルを持たない種別では null */
  label: string | null
  /** 表示する値 */
  value: string
}

const DURATION_UNIT_LABELS: Record<DurationUnit, string> = {
  day: "日",
  hour: "時間",
  minute: "分",
  second: "秒",
}

/** ラベル（note）が空の場合に種別名で代替するためのフォールバック */
const DURATION_FALLBACK_LABEL = "経過時間"
const TIME_MARKER_FALLBACK_LABEL = "時間マーカー"

const formatDurationValue = (duration: DurationValue): string =>
  `${duration.value}${DURATION_UNIT_LABELS[duration.unit]}`

/**
 * 型で網羅しているはずの分岐に未知の値が来た場合のフォールバック。
 *
 * `content` はバックエンドから JSON スカラーとして届くため、バックエンドに種別が
 * 追加されるとフロントの型より先に未知の値が流れてくる。その場合でも画面全体を
 * 落とさず、記録された内容だけは読める状態を保つ。
 */
const fallbackValue = (content: unknown): string => JSON.stringify(content) ?? ""

const formatParameterValue = (value: ParameterValue): string => {
  switch (value.type) {
    case "text":
      return value.value
    case "quantity":
      return `${value.amount}${value.unit}`
    default:
      return fallbackValue(value)
  }
}

/** note が空文字・空白のみの場合にフォールバックラベルを使う */
const toLabel = (note: string, fallback: string): string => (note.trim() === "" ? fallback : note)

/**
 * Parameter をラベルと値に整形する。
 *
 * 種別ごとに構造が異なる `content` を、タイムライン上で一様に並べられる
 * 「ラベル + 値」の形に正規化する。
 */
const formatParameter = (parameter: Parameter): FormattedParameter => {
  switch (parameter.parameterType) {
    case "KEY_VALUE":
      return {
        label: parameter.content.key,
        value: formatParameterValue(parameter.content.value),
      }
    case "DURATION":
      return {
        label: toLabel(parameter.content.note, DURATION_FALLBACK_LABEL),
        value: formatDurationValue(parameter.content.duration),
      }
    case "TIME_MARKER":
      return {
        label: toLabel(parameter.content.note, TIME_MARKER_FALLBACK_LABEL),
        value: `${formatDurationValue(parameter.content.at)}時点`,
      }
    case "TEXT":
      return { label: null, value: parameter.content.value }
    default:
      return { label: null, value: fallbackValue((parameter as Parameter).content) }
  }
}

export { formatParameter }
export type { FormattedParameter }
