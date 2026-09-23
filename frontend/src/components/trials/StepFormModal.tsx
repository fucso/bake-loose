import { useState, type FormEvent } from "react"

import { Button, buttonVariants } from "@/components/ui/button"
import { Dialog, DialogClose, DialogPopup, DialogTitle } from "@/components/ui/dialog"
import { fromDateTimeLocalValue, toDateTimeLocalValue } from "@/lib/datetime"
import { cn } from "@/lib/utils"

const inputClassName =
  "rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"

/** 工程の入力値。`startedAt` は JST オフセット付き ISO 8601 文字列、未設定は null */
type StepFormValues = {
  name: string
  startedAt: string | null
}

type StepFormModalProps = {
  /** モーダルの開閉状態 */
  open: boolean
  /** 開閉状態が変化したときのハンドラ（キャンセル・背景クリック・Escキー押下時にも呼ばれる） */
  onOpenChange: (open: boolean) => void
  /** モーダルの見出し */
  title: string
  /** 送信ボタンのラベル */
  submitLabel: string
  /** 工程名の初期値 */
  initialName: string
  /** 開始日時の初期値（ISO 8601 文字列）。未設定は null */
  initialStartedAt: string | null
  /** 送信処理中かどうか。入力・操作を無効化する */
  submitting: boolean
  /** 送信に失敗した場合に表示するメッセージ */
  errorMessage: string | null
  /** 入力内容の送信ハンドラ。送信中かどうかは `submitting` で伝える */
  onSubmit: (values: StepFormValues) => void | Promise<void>
}

/**
 * 工程の追加・編集で共通して使う入力モーダル。
 *
 * 追加と編集は送信先のミューテーションだけが異なり入力項目は同一のため、
 * 入力とバリデーションをこのコンポーネントに集約し、ミューテーションの実行は呼び出し元に委ねる。
 *
 * 開始日時は任意項目で、空欄は「未設定に戻す」意図として扱う（`onSubmit` には null を渡す）。
 */
const StepFormModal = ({
  open,
  onOpenChange,
  title,
  submitLabel,
  initialName,
  initialStartedAt,
  submitting,
  errorMessage,
  onSubmit,
}: StepFormModalProps) => {
  const [name, setName] = useState(initialName)
  const [startedAt, setStartedAt] = useState(() => toDateTimeLocalValue(initialStartedAt))
  const [validationError, setValidationError] = useState<string | null>(null)

  // 開いた時点の最新の値でフォームを初期化する。
  // 閉じている間に再取得で値が変わっても、次に開いたときには現在値が表示される。
  const [wasOpen, setWasOpen] = useState(open)
  if (open !== wasOpen) {
    setWasOpen(open)
    if (open) {
      setName(initialName)
      setStartedAt(toDateTimeLocalValue(initialStartedAt))
      setValidationError(null)
    }
  }

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const trimmedName = name.trim()
    if (trimmedName === "") {
      setValidationError("工程名を入力してください")
      return
    }
    setValidationError(null)

    onSubmit({ name: trimmedName, startedAt: fromDateTimeLocalValue(startedAt) })
  }

  // 入力の誤りは送信前に気づけるため、ミューテーションのエラーより優先して表示する
  const message = validationError ?? errorMessage

  // Dialog は開閉状態に加えて発生源の詳細も渡すため、宣言した引数だけに絞って伝える。
  // 送信中の閉じる操作（Esc・背景クリック）は無視する。閉じてしまうと送信結果の
  // 表示先が無くなり、失敗がユーザーに伝わらないため。
  const handleOpenChange = (nextOpen: boolean) => {
    if (submitting && !nextOpen) {
      return
    }
    onOpenChange(nextOpen)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogPopup>
        <DialogTitle>{title}</DialogTitle>
        <form onSubmit={handleSubmit} className="mt-4 flex flex-col gap-3">
          <label className="flex flex-col gap-1 text-sm">
            <span>工程名</span>
            <input
              autoFocus
              value={name}
              onChange={(event) => setName(event.target.value)}
              disabled={submitting}
              className={inputClassName}
            />
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span>開始日時</span>
            <input
              type="datetime-local"
              value={startedAt}
              onChange={(event) => setStartedAt(event.target.value)}
              disabled={submitting}
              className={inputClassName}
            />
          </label>
          {/* ラベルの読み上げに補足文が混ざらないよう、注記は label の外に置く */}
          <p className="-mt-2 text-xs text-muted-foreground">
            開始日時は任意項目です。空欄にすると未設定に戻ります。
          </p>

          {message && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}

          <div className="mt-2 flex justify-end gap-2">
            <DialogClose
              type="button"
              disabled={submitting}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              キャンセル
            </DialogClose>
            <Button type="submit" disabled={submitting}>
              {submitLabel}
            </Button>
          </div>
        </form>
      </DialogPopup>
    </Dialog>
  )
}

export { StepFormModal }
export type { StepFormModalProps, StepFormValues }
