import { Loader2 } from "lucide-react"

import { cn } from "@/lib/utils"

type LoadingSpinnerProps = {
  /** ローディング中に表示するメッセージ */
  message?: string
  className?: string
}

const LoadingSpinner = ({ message, className }: LoadingSpinnerProps) => {
  return (
    <div
      data-slot="loading-spinner"
      role="status"
      className={cn(
        "flex flex-col items-center justify-center gap-3 py-12 text-muted-foreground",
        className
      )}
    >
      <Loader2 className="size-8 animate-spin" aria-hidden="true" />
      {message && <p className="text-sm">{message}</p>}
    </div>
  )
}

export { LoadingSpinner }
export type { LoadingSpinnerProps }
