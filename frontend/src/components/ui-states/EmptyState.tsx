import type { LucideIcon } from "lucide-react"
import { Inbox } from "lucide-react"
import type { ReactNode } from "react"

import { cn } from "@/lib/utils"

type EmptyStateProps = {
  /** データが0件であることを示すメッセージ */
  message: string
  /** 表示するアイコン（デフォルト: Inbox） */
  icon?: LucideIcon
  /** メッセージの下に表示する補助アクション（例: 作成ボタン） */
  action?: ReactNode
  className?: string
}

function EmptyState({ message, icon: Icon = Inbox, action, className }: EmptyStateProps) {
  return (
    <div
      data-slot="empty-state"
      className={cn(
        "flex flex-col items-center justify-center gap-3 py-12 text-center text-muted-foreground",
        className
      )}
    >
      <Icon className="size-8" aria-hidden="true" />
      <p className="text-sm">{message}</p>
      {action}
    </div>
  )
}

export { EmptyState }
export type { EmptyStateProps }
