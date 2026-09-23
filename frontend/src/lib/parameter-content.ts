import type { DurationUnit, Parameter, ParameterContent, ParameterType } from "@/lib/trial"

/**
 * KeyValue の value の入力方式。`ParameterValue` のタグと対応する。
 *
 * バックエンドは更新時にこの区分も「種別」の一部として扱い、変更を拒否する
 * （`backend/src/domain/validators/trial/parameter_variant_validator.rs`）。
 */
type ParameterValueKind = "text" | "quantity"

/**
 * パラメーター入力フォームの状態。
 *
 * 種別を切り替えても入力済みの値を失わないよう、全種別の項目を1つの状態として保持する。
 * 数値項目は入力途中の文字列（`"1."`・空文字など）をそのまま保てるよう string で扱い、
 * 数値への変換と検証は `buildParameterContent` に集約する。
 */
type ParameterFormState = {
  parameterType: ParameterType
  /** KEY_VALUE: 項目名 */
  key: string
  /** KEY_VALUE: 値の入力方式 */
  valueKind: ParameterValueKind
  /** KEY_VALUE（text）: 値 */
  keyValueText: string
  /** KEY_VALUE（quantity）: 数量 */
  amount: string
  /** KEY_VALUE（quantity）: 単位 */
  unit: string
  /** DURATION / TIME_MARKER: 時間量 */
  time: string
  /** DURATION / TIME_MARKER: 時間量の単位 */
  timeUnit: DurationUnit
  /** DURATION / TIME_MARKER: 内容 */
  note: string
  /** TEXT: 自由記述 */
  text: string
}

/** 入力内容の組み立て結果。検証に失敗した場合は表示するメッセージを返す */
type BuildParameterContentResult =
  | { ok: true; content: ParameterContent }
  | { ok: false; message: string }

/** 種別選択の並び順。材料・配合の記録が最も多いため KEY_VALUE を先頭に置く */
const PARAMETER_TYPES: ParameterType[] = ["KEY_VALUE", "DURATION", "TIME_MARKER", "TEXT"]

/**
 * 入力フォームを組み立てられる種別かどうか。
 *
 * `content` は JSON スカラーのため、バックエンドに種別が追加されるとフロントの型より先に
 * 未知の値が届きうる。未知の種別は入力フォームに写せず、そのまま保存すると
 * バックエンドの種別一致検証で弾かれるため、編集の対象から外す。
 */
const isEditableParameter = (parameter: Parameter): boolean =>
  PARAMETER_TYPES.includes(parameter.parameterType)

/** 時間量の単位の選択肢 */
const DURATION_UNITS: DurationUnit[] = ["day", "hour", "minute", "second"]

/** 調理記録では分単位の指定が最も多いため初期値にする */
const DEFAULT_DURATION_UNIT: DurationUnit = "minute"

/** 材料の配合を数量で記録する用途が中心のため、KeyValue の初期値は数量にする */
const DEFAULT_VALUE_KIND: ParameterValueKind = "quantity"

const createEmptyFormState = (): ParameterFormState => ({
  parameterType: "KEY_VALUE",
  key: "",
  valueKind: DEFAULT_VALUE_KIND,
  keyValueText: "",
  amount: "",
  unit: "",
  time: "",
  timeUnit: DEFAULT_DURATION_UNIT,
  note: "",
  text: "",
})

/**
 * 既存パラメーターを編集フォームの初期状態に変換する。
 *
 * 未知の種別は `isEditableParameter` で編集対象から除外される想定だが、
 * 呼び出し漏れがあっても画面を落とさないよう空のフォームにフォールバックする。
 */
const toFormState = (parameter: Parameter): ParameterFormState => {
  const base = createEmptyFormState()

  switch (parameter.parameterType) {
    case "KEY_VALUE": {
      const { key, value } = parameter.content
      if (value.type === "quantity") {
        return {
          ...base,
          parameterType: "KEY_VALUE",
          key,
          valueKind: "quantity",
          amount: String(value.amount),
          unit: value.unit,
        }
      }
      return {
        ...base,
        parameterType: "KEY_VALUE",
        key,
        valueKind: "text",
        keyValueText: value.value,
      }
    }
    case "DURATION":
      return {
        ...base,
        parameterType: "DURATION",
        time: String(parameter.content.duration.value),
        timeUnit: parameter.content.duration.unit,
        note: parameter.content.note,
      }
    case "TIME_MARKER":
      return {
        ...base,
        parameterType: "TIME_MARKER",
        time: String(parameter.content.at.value),
        timeUnit: parameter.content.at.unit,
        note: parameter.content.note,
      }
    case "TEXT":
      return { ...base, parameterType: "TEXT", text: parameter.content.value }
    default:
      return base
  }
}

/**
 * 入力途中の文字列を数値に変換する。
 *
 * 空文字を `Number` は 0 と解釈するため、未入力と 0 の入力を区別できるよう
 * 数値として解釈できない場合は null を返す。NaN・Infinity も不正な値として扱う。
 */
const toNumber = (raw: string): number | null => {
  const trimmed = raw.trim()
  if (trimmed === "") {
    return null
  }
  const parsed = Number(trimmed)
  return Number.isFinite(parsed) ? parsed : null
}

const buildKeyValue = (state: ParameterFormState): BuildParameterContentResult => {
  const key = state.key.trim()
  if (key === "") {
    return { ok: false, message: "項目名を入力してください" }
  }

  if (state.valueKind === "quantity") {
    // バックエンドは 0 以下・数値でない amount を拒否するため、送信前に同じ条件で弾く
    const amount = toNumber(state.amount)
    if (amount === null || amount <= 0) {
      return { ok: false, message: "数量には0より大きい数値を入力してください" }
    }
    const unit = state.unit.trim()
    if (unit === "") {
      return { ok: false, message: "単位を入力してください" }
    }
    return { ok: true, content: { type: "key_value", key, value: { type: "quantity", amount, unit } } }
  }

  const value = state.keyValueText.trim()
  if (value === "") {
    return { ok: false, message: "値を入力してください" }
  }
  return { ok: true, content: { type: "key_value", key, value: { type: "text", value } } }
}

/** DURATION / TIME_MARKER はどちらも「時間量 + 内容」の組で、検証内容も共通 */
const buildTimed = (state: ParameterFormState): BuildParameterContentResult => {
  const isDuration = state.parameterType === "DURATION"

  // note は一覧表示でのラベルになり、空だと種別名しか手掛かりが残らないため必須にする
  const note = state.note.trim()
  if (note === "") {
    return { ok: false, message: "内容を入力してください" }
  }

  // バックエンドは負の値・数値でない値を拒否するため、送信前に同じ条件で弾く
  const value = toNumber(state.time)
  if (value === null || value < 0) {
    const label = isDuration ? "時間" : "経過時点"
    return { ok: false, message: `${label}には0以上の数値を入力してください` }
  }

  const duration = { value, unit: state.timeUnit }
  return isDuration
    ? { ok: true, content: { type: "duration", duration, note } }
    : { ok: true, content: { type: "time_marker", at: duration, note } }
}

/**
 * フォームの入力内容を `ParameterContent`（mutation に渡す JSON）へ組み立てる。
 *
 * 検証はバックエンドのドメインバリデーション
 * （`backend/src/domain/validators/trial/parameter_validator.rs`）と同じ条件を
 * 入力時点で適用し、記録操作の途中で往復してから弾かれることを避ける。
 */
const buildParameterContent = (state: ParameterFormState): BuildParameterContentResult => {
  switch (state.parameterType) {
    case "KEY_VALUE":
      return buildKeyValue(state)
    case "DURATION":
    case "TIME_MARKER":
      return buildTimed(state)
    case "TEXT": {
      const value = state.text.trim()
      if (value === "") {
        return { ok: false, message: "内容を入力してください" }
      }
      return { ok: true, content: { type: "text", value } }
    }
    default:
      return { ok: false, message: "種別を選択してください" }
  }
}

export {
  buildParameterContent,
  createEmptyFormState,
  DURATION_UNITS,
  isEditableParameter,
  PARAMETER_TYPES,
  toFormState,
}
export type { BuildParameterContentResult, ParameterFormState, ParameterValueKind }
