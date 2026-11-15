import { useState } from "react"
import { useQuery } from "urql"

import { CreateTrialSheet } from "@/components/trials/CreateTrialSheet"
import { TrialCard, type Trial } from "@/components/trials/TrialCard"
import { Button } from "@/components/ui/button"
import { EmptyState } from "@/components/ui-states/EmptyState"
import { ErrorState } from "@/components/ui-states/ErrorState"
import { LoadingSpinner } from "@/components/ui-states/LoadingSpinner"

type TrialsQueryData = {
  trialsByProject: Trial[]
}

type TrialsQueryVariables = {
  projectId: string
}

const TRIALS_QUERY = `
  query TrialsByProject($projectId: ID!) {
    trialsByProject(projectId: $projectId) {
      id
      name
      status
      completedAt
    }
  }
`

const CREATE_TRIAL_LABEL = "+ 新しい試行"
/** 空状態の作成導線。ヘッダーのボタンとアクセシブルネームが重複しないよう別ラベルにする */
const CREATE_FIRST_TRIAL_LABEL = "最初の試行を記録する"

type TrialListSectionProps = {
  /** 一覧対象のプロジェクトID */
  projectId: string
}

/**
 * プロジェクト詳細画面に表示する Trial 一覧セクション。
 *
 * 一覧の取得・状態表示と、Trial 作成ボトムシートの開閉を担う。
 */
const TrialListSection = ({ projectId }: TrialListSectionProps) => {
  const [isCreateSheetOpen, setIsCreateSheetOpen] = useState(false)
  const [{ data, fetching, error }, reexecuteQuery] = useQuery<
    TrialsQueryData,
    TrialsQueryVariables
  >({
    query: TRIALS_QUERY,
    variables: { projectId },
  })

  const refetchTrials = () => {
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  const handleCreated = () => {
    setIsCreateSheetOpen(false)
    refetchTrials()
  }

  const openCreateSheet = () => {
    setIsCreateSheetOpen(true)
  }

  const trials = data?.trialsByProject

  return (
    <section data-slot="trial-list-section">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">試行</h2>
        <Button onClick={openCreateSheet}>{CREATE_TRIAL_LABEL}</Button>
      </div>

      <div className="mt-4">
        {fetching && <LoadingSpinner message="読み込み中..." />}

        {!fetching && error && (
          <ErrorState message="試行の取得に失敗しました" onRetry={refetchTrials} />
        )}

        {!fetching && !error && trials && trials.length === 0 && (
          <EmptyState
            message="まだ試行がありません"
            action={<Button onClick={openCreateSheet}>{CREATE_FIRST_TRIAL_LABEL}</Button>}
          />
        )}

        {!fetching && !error && trials && trials.length > 0 && (
          <ul className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {trials.map((trial) => (
              <li key={trial.id}>
                <TrialCard trial={trial} projectId={projectId} />
              </li>
            ))}
          </ul>
        )}
      </div>

      <CreateTrialSheet
        projectId={projectId}
        open={isCreateSheetOpen}
        onOpenChange={setIsCreateSheetOpen}
        onCreated={handleCreated}
      />
    </section>
  )
}

export { TrialListSection }
export type { TrialListSectionProps }
