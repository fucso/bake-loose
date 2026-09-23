import { useState, type FormEvent } from "react"
import { useMutation } from "urql"

import { TrialStatusBadge } from "@/components/trials/TrialStatusBadge"
import { Button } from "@/components/ui/button"
import { formatDateTime } from "@/lib/datetime"
import { toUserFacingErrorMessage } from "@/lib/graphql-error"
import type { Trial, TrialStatus } from "@/lib/trial"

/** name は任意項目のため、未設定時に表示するフォールバック */
const FALLBACK_TRIAL_NAME = "名称未設定の試行"

type UpdateTrialData = {
  updateTrial: {
    id: string
    name: string | null
    memo: string | null
  }
}

type UpdateTrialVariables = {
  id: string
  input: {
    name: string | null
    memo: string | null
  }
}

const UPDATE_TRIAL_MUTATION = `
  mutation UpdateTrial($id: ID!, $input: UpdateTrialInput!) {
    updateTrial(id: $id, input: $input) {
      id
      name
      memo
    }
  }
`

type CompleteTrialData = {
  completeTrial: {
    id: string
    status: TrialStatus
    completedAt: string | null
  }
}

type CompleteTrialVariables = {
  id: string
}

// completedAt を省略するとバックエンドが現在時刻を設定する
const COMPLETE_TRIAL_MUTATION = `
  mutation CompleteTrial($id: ID!) {
    completeTrial(id: $id) {
      id
      status
      completedAt
    }
  }
`

const inputClassName =
  "rounded-md border border-input bg-background px-3 py-1.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-50"

type TrialHeaderProps = {
  /** 表示・編集する Trial */
  trial: Trial
  /** Trial の更新・完了に成功したときのハンドラ */
  onChanged: () => void
}

/**
 * Trial 自体の情報（name / memo / status / completedAt）の表示と操作を担う。
 *
 * 工程の記録に集中できるよう、編集フォームは常時表示せず「編集」操作で開く。
 */
const TrialHeader = ({ trial, onChanged }: TrialHeaderProps) => {
  const [isEditing, setIsEditing] = useState(false)
  const [name, setName] = useState(trial.name ?? "")
  const [memo, setMemo] = useState(trial.memo ?? "")

  const [{ fetching: updating, error: updateError }, updateTrial] = useMutation<
    UpdateTrialData,
    UpdateTrialVariables
  >(UPDATE_TRIAL_MUTATION)
  const [{ fetching: completing, error: completeError }, completeTrial] = useMutation<
    CompleteTrialData,
    CompleteTrialVariables
  >(COMPLETE_TRIAL_MUTATION)

  const startEditing = () => {
    setName(trial.name ?? "")
    setMemo(trial.memo ?? "")
    setIsEditing(true)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const trimmedName = name.trim()
    const trimmedMemo = memo.trim()

    // name / memo はいずれも任意項目。空欄は「未設定に戻す」意図として
    // null を送り、バックエンド側の値をクリアする
    const result = await updateTrial({
      id: trial.id,
      input: {
        name: trimmedName === "" ? null : trimmedName,
        memo: trimmedMemo === "" ? null : trimmedMemo,
      },
    })

    if (!result.error) {
      setIsEditing(false)
      onChanged()
    }
  }

  const handleComplete = async () => {
    const result = await completeTrial({ id: trial.id })
    if (!result.error) {
      onChanged()
    }
  }

  const updateErrorMessage = toUserFacingErrorMessage(updateError, "試行の更新に失敗しました")
  const completeErrorMessage = toUserFacingErrorMessage(completeError, "試行の完了に失敗しました")

  if (isEditing) {
    return (
      <form onSubmit={handleSubmit} className="flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm">
          <span>試行名</span>
          <input
            autoFocus
            value={name}
            onChange={(event) => setName(event.target.value)}
            disabled={updating}
            className={inputClassName}
          />
        </label>

        <label className="flex flex-col gap-1 text-sm">
          <span>メモ</span>
          <textarea
            rows={3}
            value={memo}
            onChange={(event) => setMemo(event.target.value)}
            disabled={updating}
            className={inputClassName}
          />
        </label>

        {updateErrorMessage && <p className="text-sm text-destructive">{updateErrorMessage}</p>}

        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={updating}
            onClick={() => setIsEditing(false)}
          >
            キャンセル
          </Button>
          <Button type="submit" disabled={updating}>
            保存
          </Button>
        </div>
      </form>
    )
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-2xl font-bold break-words">{trial.name ?? FALLBACK_TRIAL_NAME}</h1>
        <TrialStatusBadge status={trial.status} />
      </div>

      {trial.memo && <p className="text-sm break-words text-muted-foreground">{trial.memo}</p>}

      {trial.completedAt && (
        <p className="text-sm text-muted-foreground">
          完了日時: {formatDateTime(trial.completedAt)}
        </p>
      )}

      {completeErrorMessage && <p className="text-sm text-destructive">{completeErrorMessage}</p>}

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" size="sm" onClick={startEditing}>
          編集
        </Button>
        {trial.status === "IN_PROGRESS" && (
          <Button size="sm" disabled={completing} onClick={handleComplete}>
            完了にする
          </Button>
        )}
      </div>
    </div>
  )
}

export { TrialHeader }
export type { TrialHeaderProps }
