import { useState } from "react"
import { Link, useParams } from "react-router-dom"
import { useQuery } from "urql"

import { StepTimeline } from "@/components/trials/StepTimeline"
import { TrialHeader } from "@/components/trials/TrialHeader"
import { ErrorState } from "@/components/ui-states/ErrorState"
import { LoadingSpinner } from "@/components/ui-states/LoadingSpinner"
import type { Trial } from "@/lib/trial"

type TrialQueryData = {
  trial: Trial | null
}

type TrialQueryVariables = {
  id: string
}

// 工程・パラメーターは1画面で俯瞰するため、追加のラウンドトリップを避けて一度に取得する
const TRIAL_QUERY = `
  query Trial($id: ID!) {
    trial(id: $id) {
      id
      projectId
      name
      memo
      status
      completedAt
      steps {
        id
        name
        position
        startedAt
        completedAt
        isCompleted
        parameters {
          id
          parameterType
          content
        }
      }
    }
  }
`

const HYPHENATED = "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
// バックエンド（uuid クレートの parse_str）が受け付ける表記: ハイフン区切り・ハイフンなし・
// 波括弧つき・`urn:uuid:` つき（urn の接頭辞は小文字のみ）。それ以外は UUID として扱わない
const UUID_FORMS = [
  new RegExp(`^(${HYPHENATED})$`),
  new RegExp(`^\\{(${HYPHENATED})\\}$`),
  new RegExp(`^urn:uuid:(${HYPHENATED})$`),
]
const SIMPLE_UUID = /^[0-9a-fA-F]{32}$/

/**
 * URL の trialId をバックエンドが返す正規形（小文字・ハイフン区切り）へそろえる。
 * UUID として解釈できない値は null を返す。
 */
const normalizeTrialId = (raw: string | undefined): string | null => {
  if (raw === undefined) {
    return null
  }
  if (SIMPLE_UUID.test(raw)) {
    const h = raw.toLowerCase()
    return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`
  }
  for (const form of UUID_FORMS) {
    const match = form.exec(raw)
    if (match) {
      return match[1].toLowerCase()
    }
  }
  return null
}

/**
 * Trial 詳細（記録ワークベンチ）。
 *
 * 1つの Trial に紐づく工程・パラメーターを時系列で俯瞰し、Trial 自体の編集・完了と
 * 各工程へのパラメーターの記録（追加・編集・削除）を行う。
 * 工程そのものの記録操作（追加・編集・完了）は別途実装する。
 */
const TrialDetailPage = () => {
  const { id, trialId } = useParams<{ id: string; trialId: string }>()
  // URL は手で書き換えられる。UUID でない値はバックエンドが GraphQL エラーを返すだけで
  // 再試行しても成功しないため、クエリを送らず「見つからない」として扱う
  const normalizedTrialId = normalizeTrialId(trialId)
  const isValidTrialId = normalizedTrialId !== null

  const [{ data, fetching, error }, reexecuteQuery] = useQuery<
    TrialQueryData,
    TrialQueryVariables
  >({
    query: TRIAL_QUERY,
    variables: { id: normalizedTrialId ?? "" },
    pause: !isValidTrialId,
  })

  const refetch = () => {
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  // 別の Trial の URL へ遷移した直後は前の Trial のデータが残るため、表示対象の ID と
  // 一致するものだけを使う。一致しない間は初回取得と同じくスピナーを出す
  const isCurrentTrial = (candidate: Trial | null | undefined): candidate is Trial =>
    candidate?.id.toLowerCase() === normalizedTrialId

  // urql はエラー結果を受け取ると data を空にする。再取得に失敗しても表示済みの内容と
  // 工程カードの開閉状態を保てるよう、直近に取得できた Trial を保持しておく
  const [lastTrial, setLastTrial] = useState<Trial | null>(null)
  if (isCurrentTrial(data?.trial) && data.trial !== lastTrial) {
    setLastTrial(data.trial)
  }

  // data.trial が null（削除された等）の場合は保持している内容を使わない
  const trial = isCurrentTrial(data?.trial)
    ? data.trial
    : data?.trial === null
      ? null
      : isCurrentTrial(lastTrial)
        ? lastTrial
        : null
  // 不正な trialId の場合はクエリを実行していないため、応答を待たずに見つからない扱いにする
  const isNotFound =
    !isValidTrialId || (!fetching && !error && data !== undefined && data.trial === null)

  // Trial を取得できている場合は Trial 自身の projectId を戻り先にする。
  // URL の :id が実際の所属プロジェクトと食い違っていても正しい一覧へ戻れる。
  const backTo = `/projects/${trial?.projectId ?? id}`

  return (
    <div>
      <Link
        to={backTo}
        className="text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        ← 戻る
      </Link>

      <div className="mt-6">
        {/*
          スピナーは初回取得時だけに出す。更新後の再取得で内容を差し替えてしまうと
          工程カードの開閉状態やスクロール位置がリセットされ、記録操作を妨げるため。
        */}
        {fetching && !trial && <LoadingSpinner message="読み込み中..." />}

        {/* 表示済みの内容があっても再取得に失敗した場合は、内容を残したまま失敗を知らせる */}
        {!fetching && error && (
          <ErrorState message="試行の取得に失敗しました" onRetry={refetch} />
        )}

        {isNotFound && <ErrorState message="指定された試行が見つかりません" />}

        {/* key で Trial ごとに作り直し、編集中のフォームや完了済みフラグを別の Trial へ持ち越さない */}
        {trial && (
          <div key={trial.id} className="flex flex-col gap-6">
            <TrialHeader trial={trial} onChanged={refetch} />
            <StepTimeline
              trialId={trial.id}
              steps={trial.steps}
              trialStatus={trial.status}
              onChanged={refetch}
            />
          </div>
        )}
      </div>
    </div>
  )
}

export default TrialDetailPage
