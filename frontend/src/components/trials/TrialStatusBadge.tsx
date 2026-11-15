import { cn } from "@/lib/utils"
import type { TrialStatus } from "@/lib/trial"

const STATUS_LABELS: Record<TrialStatus, string> = {
  IN_PROGRESS: "記録中",
  COMPLETED: "完了",
}

const STATUS_CLASSES: Record<TrialStatus, string> = {
  IN_PROGRESS: "bg-primary/10 text-primary",
  COMPLETED: "bg-muted text-muted-foreground",
}

type TrialStatusBadgeProps = {
  /** 表示する Trial のステータス */
  status: TrialStatus
  className?: string
}

const TrialStatusBadge = ({ status, className }: TrialStatusBadgeProps) => {
  return (
    <span
      data-slot="trial-status-badge"
      className={cn(
        "inline-flex shrink-0 items-center rounded-full px-2 py-0.5 text-xs font-medium",
        STATUS_CLASSES[status],
        className
      )}
    >
      {STATUS_LABELS[status]}
    </span>
  )
}

export { TrialStatusBadge }
export type { TrialStatusBadgeProps }
