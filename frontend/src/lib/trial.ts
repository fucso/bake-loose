/**
 * Trial / Step / Parameter の GraphQL レスポンス型。
 *
 * バックエンドの `backend/src/presentation/graphql/types/trial.rs` と
 * `backend/src/domain/models/parameter.rs` に対応する。
 * GraphQL の enum は SCREAMING_SNAKE_CASE、`content`（JSON スカラー）内の
 * タグは serde の snake_case で返る点に注意する。
 */

/** Trial のステータス */
type TrialStatus = "IN_PROGRESS" | "COMPLETED"

/** 時間量の単位 */
type DurationUnit = "day" | "hour" | "minute" | "second"

/** 時間量（値 + 単位） */
type DurationValue = {
  value: number
  unit: DurationUnit
}

/** KeyValue パラメーターの value 部分 */
type ParameterValue =
  | { type: "text"; value: string }
  | { type: "quantity"; amount: number; unit: string }

/** キーと値の組（例: 強力粉 300g） */
type KeyValueContent = {
  type: "key_value"
  key: string
  value: ParameterValue
}

/** 経過時間（例: 一次発酵 90分） */
type DurationContent = {
  type: "duration"
  duration: DurationValue
  note: string
}

/** 時間マーカー（例: 焼成開始から30分時点） */
type TimeMarkerContent = {
  type: "time_marker"
  at: DurationValue
  note: string
}

/** 自由記述 */
type TextContent = {
  type: "text"
  value: string
}

/**
 * Parameter。
 *
 * `parameterType` は `content` の JSON を解析せずに種別を判別するためにバックエンドが
 * 公開している列挙子で、`content` のバリアントと1対1で対応する。
 * これを判別子にすることで `content` の型が絞り込まれる。
 */
type Parameter =
  | { id: string; parameterType: "KEY_VALUE"; content: KeyValueContent }
  | { id: string; parameterType: "DURATION"; content: DurationContent }
  | { id: string; parameterType: "TIME_MARKER"; content: TimeMarkerContent }
  | { id: string; parameterType: "TEXT"; content: TextContent }

/** Parameter の種別 */
type ParameterType = Parameter["parameterType"]

/**
 * Parameter の内容。
 *
 * 記録操作の mutation（`addParameter` / `updateParameter`）に JSON スカラーとして渡す値でもある。
 */
type ParameterContent = KeyValueContent | DurationContent | TimeMarkerContent | TextContent

/** Trial 内の工程 */
type Step = {
  id: string
  name: string
  /** Trial 内での位置（0始まり） */
  position: number
  startedAt: string | null
  completedAt: string | null
  isCompleted: boolean
  parameters: Parameter[]
}

/** 試行 */
type Trial = {
  id: string
  projectId: string
  name: string | null
  memo: string | null
  status: TrialStatus
  completedAt: string | null
  steps: Step[]
}

export type {
  DurationUnit,
  DurationValue,
  Parameter,
  ParameterContent,
  ParameterType,
  ParameterValue,
  Step,
  Trial,
  TrialStatus,
}
