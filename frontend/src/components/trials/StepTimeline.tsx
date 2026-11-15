import { useEffect, useRef } from "react"

import { StepCard } from "@/components/trials/StepCard"
import { EmptyState } from "@/components/ui-states/EmptyState"
import type { Step, TrialStatus } from "@/lib/trial"

type StepTimelineProps = {
  /** 対象の Trial ID */
  trialId: string
  /** 表示する工程一覧（順序は問わない。position 昇順に整列して表示する） */
  steps: Step[]
  /** Trial のステータス。完了済み Trial は全工程を畳んだ俯瞰表示にする */
  trialStatus: TrialStatus
  /** パラメーターの記録操作に成功したときのハンドラ */
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
 * 工程を position 昇順のタイムラインで表示する。
 *
 * 記録中の工程だけを開いた状態で表示し、完了済みの工程は畳むことで
 * モバイルでのスクロール量を抑える。記録中の工程は初期表示時に画面内へスクロールする。
 */
const StepTimeline = ({ trialId, steps, trialStatus, onChanged }: StepTimelineProps) => {
  const sortedSteps = [...steps].sort((a, b) => a.position - b.position)
  const currentStepId = findCurrentStepId(sortedSteps, trialStatus)
  const currentStepRef = useRef<HTMLLIElement>(null)

  useEffect(() => {
    // jsdom など scrollIntoView を実装しない環境があるため存在確認してから呼ぶ
    // 既に視界にある場合は動かさないよう nearest を使い、不要なスクロールを避ける
    currentStepRef.current?.scrollIntoView?.({ block: "nearest" })
  }, [currentStepId])

  if (sortedSteps.length === 0) {
    return <EmptyState message="まだ工程が記録されていません" />
  }

  return (
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
              // バックエンドは完了済みの Trial・工程へのパラメーター操作を拒否するため、
              // 記録できる工程にだけ操作 UI を出す
              editable={trialStatus === "IN_PROGRESS" && !step.isCompleted}
              onChanged={onChanged}
            />
          </li>
        )
      })}
    </ol>
  )
}

export { StepTimeline }
export type { StepTimelineProps }
