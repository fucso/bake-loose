import { useEffect, useRef } from "react"

import { AddStepButton } from "@/components/trials/AddStepButton"
import { StepCard } from "@/components/trials/StepCard"
import { EmptyState } from "@/components/ui-states/EmptyState"
import type { Step, TrialStatus } from "@/lib/trial"

type StepTimelineProps = {
  /** 工程が属する Trial の ID */
  trialId: string
  /** 表示する工程一覧（順序は問わない。position 昇順に整列して表示する） */
  steps: Step[]
  /** Trial のステータス。完了済み Trial は全工程を畳んだ俯瞰表示にする */
  trialStatus: TrialStatus
  /** 工程の記録操作に成功したときのハンドラ */
  onChanged: () => void
}

/** position 昇順で最初の未完了工程を「記録中の工程」とみなす */
const findCurrentStepId = (sortedSteps: Step[], trialStatus: TrialStatus): string | null => {
  if (trialStatus === "COMPLETED") {
    return null
  }
  return sortedSteps.find((step) => !step.isCompleted)?.id ?? null
}

/**
 * 工程を position 昇順のタイムラインで表示し、工程の記録操作の入り口を提供する。
 *
 * 記録中の工程だけを開いた状態で表示し、完了済みの工程は畳むことで
 * モバイルでのスクロール量を抑える。記録中の工程は初期表示時に画面内へスクロールする。
 *
 * 記録操作は完了済み Trial では行えないため、その場合は操作を表示しない。
 */
const StepTimeline = ({ trialId, steps, trialStatus, onChanged }: StepTimelineProps) => {
  const sortedSteps = [...steps].sort((a, b) => a.position - b.position)
  const currentStepId = findCurrentStepId(sortedSteps, trialStatus)
  const currentStepRef = useRef<HTMLLIElement>(null)
  // 完了済み Trial には工程を追加できず、既存の工程も編集・完了できない
  const canRecord = trialStatus === "IN_PROGRESS"

  useEffect(() => {
    // jsdom など scrollIntoView を実装しない環境があるため存在確認してから呼ぶ
    // 既に視界にある場合は動かさないよう nearest を使い、不要なスクロールを避ける
    currentStepRef.current?.scrollIntoView?.({ block: "nearest" })
  }, [currentStepId])

  return (
    <div className="flex flex-col gap-3">
      {sortedSteps.length === 0 ? (
        <EmptyState message="まだ工程が記録されていません" />
      ) : (
        <ol data-slot="step-timeline" className="flex flex-col gap-3">
          {sortedSteps.map((step) => {
            const isCurrent = step.id === currentStepId
            return (
              <li
                key={step.id}
                ref={isCurrent ? currentStepRef : undefined}
                aria-current={isCurrent ? "step" : undefined}
              >
                <StepCard
                  trialId={trialId}
                  step={step}
                  isCurrent={isCurrent}
                  // 記録の焦点である未完了工程だけを開き、完了済みは畳んで一覧性を優先する
                  defaultExpanded={trialStatus !== "COMPLETED" && !step.isCompleted}
                  // 完了済みの工程は編集・完了ができない（バックエンドの不変条件に合わせる）
                  canRecord={canRecord && !step.isCompleted}
                  onChanged={onChanged}
                />
              </li>
            )
          })}
        </ol>
      )}

      {/* 追加された工程は position の末尾に積まれるため、入り口もタイムラインの末尾に置く */}
      {canRecord && (
        <div>
          <AddStepButton trialId={trialId} onAdded={onChanged} />
        </div>
      )}
    </div>
  )
}

export { StepTimeline }
export type { StepTimelineProps }
