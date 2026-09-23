import { Link } from "react-router-dom"

import { formatDateTime } from "@/lib/date"
import { cn } from "@/lib/utils"

/** バックエンドの GraphQL enum `TrialStatus` に対応する値 */
type TrialStatus = "IN_PROGRESS" | "COMPLETED"

/** Trial 一覧カードの表示に必要な最小限のフィールド */
type Trial = {
  id: string
  name: string | null
  status: TrialStatus
  completedAt: string | null
}

/** name が未設定の Trial に表示するフォールバックラベル */
const UNNAMED_TRIAL_LABEL = "名称未設定の試行"

const STATUS_LABELS: Record<TrialStatus, string> = {
  IN_PROGRESS: "進行中",
  COMPLETED: "完了",
}

const STATUS_BADGE_CLASSES: Record<TrialStatus, string> = {
  IN_PROGRESS: "bg-primary/10 text-primary",
  COMPLETED: "bg-muted text-muted-foreground",
}

type TrialCardProps = {
  /** 表示する Trial */
  trial: Trial
  /** 詳細画面への遷移先 URL を組み立てるためのプロジェクトID */
  projectId: string
  className?: string
}

/**
 * Trial 1件をカードとして表示し、Trial 詳細画面へのリンクにする。
 *
 * 遷移先 `/projects/:id/trials/:trialId` の画面本体は別 Issue のスコープのため、
 * 本コンポーネントはリンクの提供までを担う。
 */
const TrialCard = ({ trial, projectId, className }: TrialCardProps) => {
  const completedAt = trial.completedAt ? formatDateTime(trial.completedAt) : null

  return (
    <Link
      to={`/projects/${projectId}/trials/${trial.id}`}
      data-slot="trial-card"
      className={cn(
        "flex flex-col gap-2 rounded-lg border border-border bg-card p-4 text-card-foreground transition-colors hover:bg-muted",
        className
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className={cn("font-medium", !trial.name && "text-muted-foreground")}>
          {trial.name || UNNAMED_TRIAL_LABEL}
        </span>
        <span
          className={cn(
            "shrink-0 rounded-full px-2 py-0.5 text-xs font-medium",
            STATUS_BADGE_CLASSES[trial.status]
          )}
        >
          {STATUS_LABELS[trial.status]}
        </span>
      </div>
      {completedAt && <span className="text-xs text-muted-foreground">完了 {completedAt}</span>}
    </Link>
  )
}

export { TrialCard, UNNAMED_TRIAL_LABEL }
export type { Trial, TrialCardProps, TrialStatus }
