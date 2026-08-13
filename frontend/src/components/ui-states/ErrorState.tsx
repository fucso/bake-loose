import { CircleAlert, RefreshCw } from "lucide-react"

import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

type ErrorStateProps = {
  /** 表示するエラーメッセージ */
  message: string
  /** 再試行ハンドラ。指定しない場合は再試行ボタンを表示しない */
  onRetry?: () => void
  /** 再試行ボタンのラベル */
  retryLabel?: string
  className?: string
}

function ErrorState({ message, onRetry, retryLabel = "再試行", className }: ErrorStateProps) {
  return (
    <div
      data-slot="error-state"
      role="alert"
      className={cn("flex flex-col items-center justify-center gap-3 py-12 text-center", className)}
    >
      <CircleAlert className="size-8 text-destructive" aria-hidden="true" />
      <p className="text-sm text-destructive">{message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          <RefreshCw className="size-3.5" aria-hidden="true" />
          {retryLabel}
        </Button>
      )}
    </div>
  )
}

export { ErrorState }
export type { ErrorStateProps }
