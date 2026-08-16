import { useState, type FormEvent } from "react"
import { useMutation } from "urql"

import { Button, buttonVariants } from "@/components/ui/button"
import { Dialog, DialogClose, DialogPopup, DialogTitle } from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

type CreateProjectData = {
  createProject: {
    id: string
    name: string
  }
}

type CreateProjectVariables = {
  input: {
    name: string
  }
}

const CREATE_PROJECT_MUTATION = `
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      id
      name
    }
  }
`

type CreateProjectModalProps = {
  /** モーダルの開閉状態 */
  open: boolean
  /** 開閉状態が変化したときのハンドラ（キャンセル・背景クリック・Escキー押下時にも呼ばれる） */
  onOpenChange: (open: boolean) => void
  /** プロジェクト作成に成功したときのハンドラ */
  onCreated: () => void
}

function CreateProjectModal({ open, onOpenChange, onCreated }: CreateProjectModalProps) {
  const [name, setName] = useState("")
  const [validationError, setValidationError] = useState<string | null>(null)
  const [{ fetching, error }, createProject] = useMutation<
    CreateProjectData,
    CreateProjectVariables
  >(CREATE_PROJECT_MUTATION)

  const resetForm = () => {
    setName("")
    setValidationError(null)
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
    if (!trimmedName) {
      setValidationError("プロジェクト名を入力してください")
      return
    }
    setValidationError(null)

    const result = await createProject({ input: { name: trimmedName } })
    if (!result.error) {
      resetForm()
      onCreated()
    }
  }

  // バックエンドの GraphQL エラーメッセージ（プロジェクト名の重複・文字数超過など）は
  // presentation層（backend/src/presentation/graphql/error.rs）で既にユーザー向けに
  // 変換済みのため、そのまま表示する。GraphQL エラーが無い場合（ネットワークエラー等）は
  // 汎用メッセージにフォールバックする。
  const errorMessage =
    validationError ??
    (error ? (error.graphQLErrors[0]?.message ?? "プロジェクトの作成に失敗しました") : null)

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogPopup>
        <DialogTitle>新規プロジェクト作成</DialogTitle>
        <form onSubmit={handleSubmit} className="mt-4 flex flex-col gap-3">
          <label className="flex flex-col gap-1 text-sm">
            <span>プロジェクト名</span>
            <input
              autoFocus
              value={name}
              onChange={(event) => setName(event.target.value)}
              disabled={fetching}
              className="rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"
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

export { CreateProjectModal }
