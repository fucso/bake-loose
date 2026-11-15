import { ChevronDown } from "lucide-react"
import { useState } from "react"

import { StepParameterList } from "@/components/trials/StepParameterList"
import { formatDateTime } from "@/lib/datetime"
import type { Step } from "@/lib/trial"
import { cn } from "@/lib/utils"

type StepCardProps = {
  /** 対象の Trial ID */
  trialId: string
  /** 表示する工程 */
  step: Step
  /** 記録中の工程（position 順で最初の未完了工程）かどうか */
  isCurrent: boolean
  /**
   * 初期表示で展開するかどうか。
   * 以降の開閉はユーザー操作を優先するため、この値の変化では同期しない。
   */
  defaultExpanded: boolean
  /** この工程にパラメーターを記録できるか */
  editable: boolean
  /** パラメーターの記録操作に成功したときのハンドラ */
  onChanged: () => void
}

/**
 * 工程の状態ラベル。
 *
 * Trial のステータス（記録中 / 完了）と一目で区別できるよう、工程側は別の語を使う。
 */
const stepStateLabel = (step: Step, isCurrent: boolean): string => {
  if (step.isCompleted) {
    return "完了"
  }
  return isCurrent ? "進行中" : "未着手"
}

/**
 * 工程1件をカードで表示する。
 *
 * 1画面に全工程が並ぶため、本文（パラメーター）は折りたたみ可能にし、
 * 折りたたみ時もヘッダだけで進捗を把握できるようサマリーを常に表示する。
 */
const StepCard = ({
  trialId,
  step,
  isCurrent,
  defaultExpanded,
  editable,
  onChanged,
}: StepCardProps) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded)
  const contentId = `step-parameters-${step.id}`

  return (
    <div
      data-slot="step-card"
      className={cn(
        "rounded-lg border bg-card text-card-foreground",
        isCurrent ? "border-primary" : "border-border"
      )}
    >
      <button
        type="button"
        onClick={() => setIsExpanded((expanded) => !expanded)}
        aria-expanded={isExpanded}
        aria-controls={contentId}
        className="flex w-full items-start gap-3 p-4 text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
      >
        <span className="mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium text-muted-foreground">
          {step.position + 1}
        </span>

        <span className="flex min-w-0 flex-1 flex-col gap-1">
          <span className="flex flex-wrap items-center gap-2">
            <span className="font-medium break-words">{step.name}</span>
            <span className="text-xs text-muted-foreground">
              {stepStateLabel(step, isCurrent)}
            </span>
          </span>

          <span className="text-xs text-muted-foreground">
            パラメーター {step.parameters.length}件
            {step.startedAt && ` ・ 開始 ${formatDateTime(step.startedAt)}`}
            {step.completedAt && ` ・ 完了 ${formatDateTime(step.completedAt)}`}
          </span>
        </span>

        <ChevronDown
          aria-hidden="true"
          className={cn(
            "mt-0.5 size-4 shrink-0 text-muted-foreground transition-transform",
            isExpanded && "rotate-180"
          )}
        />
      </button>

      <div id={contentId} hidden={!isExpanded} className="border-t border-border px-4 py-3 pl-13">
        <StepParameterList
          trialId={trialId}
          step={step}
          editable={editable}
          onChanged={onChanged}
        />
      </div>
    </div>
  )
}

export { StepCard }
export type { StepCardProps }
