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

/**
 * Trial 詳細（記録ワークベンチ）。
 *
 * 1つの Trial に紐づく工程・パラメーターを時系列で俯瞰し、Trial 自体の編集・完了と
 * 各工程へのパラメーターの記録（追加・編集・削除）を行う。
 * 工程そのものの記録操作（追加・編集・完了）は別途実装する。
 */
const TrialDetailPage = () => {
  const { id, trialId } = useParams<{ id: string; trialId: string }>()

  const [{ data, fetching, error }, reexecuteQuery] = useQuery<
    TrialQueryData,
    TrialQueryVariables
  >({
    query: TRIAL_QUERY,
    variables: { id: trialId ?? "" },
    pause: !trialId,
  })

  const refetch = () => {
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  const trial = data?.trial ?? null
  // trialId が無い場合はクエリを実行していないため、応答を待たずに見つからない扱いにする
  const isNotFound = !trialId || (!fetching && !error && data !== undefined && trial === null)

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

        {!fetching && error && (
          <ErrorState message="試行の取得に失敗しました" onRetry={refetch} />
        )}

        {!fetching && !error && isNotFound && (
          <ErrorState message="指定された試行が見つかりません" />
        )}

        {!error && trial && (
          <div className="flex flex-col gap-6">
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
