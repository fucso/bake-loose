import { useState, type FormEvent } from "react"
import { useMutation } from "urql"

import { Button, buttonVariants } from "@/components/ui/button"
import { Dialog, DialogClose, DialogPopup, DialogTitle } from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

type CreateTrialData = {
  createTrial: {
    id: string
  }
}

type CreateTrialVariables = {
  input: {
    projectId: string
    name: string | null
    memo: string | null
  }
}

// 作成結果は画面に直接反映せず、呼び出し元での一覧再取得に委ねるため id のみ取得する
const CREATE_TRIAL_MUTATION = `
  mutation CreateTrial($input: CreateTrialInput!) {
    createTrial(input: $input) {
      id
    }
  }
`

/** 作成の失敗理由が GraphQL エラーとして得られなかった場合に表示する汎用メッセージ */
const FALLBACK_ERROR_MESSAGE = "試行の作成に失敗しました"

/**
 * 画面下部からせり上がるボトムシートとして表示するための DialogPopup 上書きクラス。
 *
 * 調理中の片手操作を想定し、モーダル中央配置ではなく親指の届く下端に寄せる。
 * iOS のホームインジケーターに被らないよう下部の safe-area 分を追加で確保する。
 */
const BOTTOM_SHEET_CLASSES = cn(
  "top-auto bottom-0 w-full max-w-md translate-y-0 rounded-t-2xl rounded-b-none",
  "pb-[calc(1.5rem+env(safe-area-inset-bottom))]",
  "transition-transform duration-200 data-[ending-style]:translate-y-full data-[starting-style]:translate-y-full"
)

type CreateTrialSheetProps = {
  /** 作成する Trial が属するプロジェクトID */
  projectId: string
  /** シートの開閉状態 */
  open: boolean
  /** 開閉状態が変化したときのハンドラ（キャンセル・背景クリック・Escキー押下時にも呼ばれる） */
  onOpenChange: (open: boolean) => void
  /** Trial の作成に成功したときのハンドラ */
  onCreated: () => void
}

/**
 * Trial を新規作成するボトムシート。
 *
 * name・memo はいずれも任意入力のため、未入力のままでも作成できる。
 * 未入力の項目は空文字ではなく null を送り、バックエンド上も「未設定」として扱わせる。
 */
const CreateTrialSheet = ({
  projectId,
  open,
  onOpenChange,
  onCreated,
}: CreateTrialSheetProps) => {
  const [name, setName] = useState("")
  const [memo, setMemo] = useState("")
  // useMutation が保持する error はシートを閉じてもクリアされず、再度開いたときに
  // 前回の失敗メッセージが残ってしまうため、表示用のエラーは自前の state で管理する
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [{ fetching }, createTrial] = useMutation<CreateTrialData, CreateTrialVariables>(
    CREATE_TRIAL_MUTATION
  )

  const resetForm = () => {
    setName("")
    setMemo("")
    setErrorMessage(null)
  }

  const handleOpenChange = (nextOpen: boolean) => {
    if (!nextOpen) {
      resetForm()
    }
    onOpenChange(nextOpen)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const trimmedName = name.trim()
    const trimmedMemo = memo.trim()

    const result = await createTrial({
      input: {
        projectId,
        name: trimmedName || null,
        memo: trimmedMemo || null,
      },
    })

    if (result.error) {
      // バックエンドの GraphQL エラーメッセージは presentation 層
      // （backend/src/presentation/graphql/error.rs）でユーザー向けに変換済みのため、そのまま表示する。
      // GraphQL エラーが無い場合（ネットワークエラー等）は汎用メッセージにフォールバックする。
      setErrorMessage(result.error.graphQLErrors[0]?.message ?? FALLBACK_ERROR_MESSAGE)
      return
    }

    resetForm()
    onCreated()
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogPopup className={BOTTOM_SHEET_CLASSES}>
        <DialogTitle>新しい試行</DialogTitle>
        <form onSubmit={handleSubmit} className="mt-4 flex flex-col gap-3">
          <label className="flex flex-col gap-1 text-sm">
            <span>試行名（任意）</span>
            <input
              autoFocus
              value={name}
              onChange={(event) => setName(event.target.value)}
              disabled={fetching}
              className="rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"
            />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span>メモ（任意）</span>
            <textarea
              rows={3}
              value={memo}
              onChange={(event) => setMemo(event.target.value)}
              disabled={fetching}
              className="resize-none rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"
            />
          </label>
          {errorMessage && <p className="text-sm text-destructive">{errorMessage}</p>}
          <div className="mt-2 flex justify-end gap-2">
            <DialogClose
              type="button"
              disabled={fetching}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              キャンセル
            </DialogClose>
            <Button type="submit" disabled={fetching}>
              作成
            </Button>
          </div>
        </form>
      </DialogPopup>
    </Dialog>
  )
}

export { CreateTrialSheet }
export type { CreateTrialSheetProps }
