import { useRef, useState, type FormEvent } from "react"
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

  // 失敗メッセージは urql の保持するエラーではなく自前の state で持つ。
  // urql のエラーは次のミューテーションまで残り、キャンセル後に再度開いた編集フォームへ
  // 前回の失敗が再表示されてしまうため。
  const [updateErrorMessage, setUpdateErrorMessage] = useState<string | null>(null)
  const [completeErrorMessage, setCompleteErrorMessage] = useState<string | null>(null)
  // 完了に成功してから再取得で status が COMPLETED に変わるまでの間、完了操作を無効にする
  const [completeRequested, setCompleteRequested] = useState(false)
  // disabled は次の描画まで効かないため、同一 tick の二重送信は ref で防ぐ
  const inFlightRef = useRef(false)

  const [{ fetching: updating }, updateTrial] = useMutation<
    UpdateTrialData,
    UpdateTrialVariables
  >(UPDATE_TRIAL_MUTATION)
  const [{ fetching: completing }, completeTrial] = useMutation<
    CompleteTrialData,
    CompleteTrialVariables
  >(COMPLETE_TRIAL_MUTATION)

  const startEditing = () => {
    setName(trial.name ?? "")
    setMemo(trial.memo ?? "")
    setUpdateErrorMessage(null)
    setIsEditing(true)
  }

  const cancelEditing = () => {
    setUpdateErrorMessage(null)
    setIsEditing(false)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (inFlightRef.current) {
      return
    }

    const trimmedName = name.trim()
    const trimmedMemo = memo.trim()

    // name / memo はいずれも任意項目。空欄は「未設定に戻す」意図として
    // null を送り、バックエンド側の値をクリアする
    inFlightRef.current = true
    try {
      const result = await updateTrial({
        id: trial.id,
        input: {
          name: trimmedName === "" ? null : trimmedName,
          memo: trimmedMemo === "" ? null : trimmedMemo,
        },
      })

      if (result.error) {
        setUpdateErrorMessage(toUserFacingErrorMessage(result.error, "試行の更新に失敗しました"))
        return
      }
      setUpdateErrorMessage(null)
      setIsEditing(false)
      onChanged()
    } finally {
      inFlightRef.current = false
    }
  }

  const handleComplete = async () => {
    if (inFlightRef.current) {
      return
    }
    inFlightRef.current = true
    try {
      const result = await completeTrial({ id: trial.id })
      if (result.error) {
        setCompleteErrorMessage(toUserFacingErrorMessage(result.error, "試行の完了に失敗しました"))
        return
      }
      setCompleteErrorMessage(null)
      setCompleteRequested(true)
      onChanged()
    } finally {
      inFlightRef.current = false
    }
  }

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

        {updateErrorMessage && (
          <p role="alert" className="text-sm text-destructive">
            {updateErrorMessage}
          </p>
        )}

        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={updating}
            onClick={cancelEditing}
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

      {completeErrorMessage && (
        <p role="alert" className="text-sm text-destructive">
          {completeErrorMessage}
        </p>
      )}

      <div className="flex flex-wrap gap-2">
        {/* 完了済みの Trial はバックエンドが更新を拒否するため、編集操作を出さない */}
        {trial.status === "IN_PROGRESS" && (
          <>
            {/* 完了に成功した後は、再取得で status が反映されるまでの間も編集させない */}
            {!completeRequested && (
              <Button variant="outline" size="sm" onClick={startEditing}>
                編集
              </Button>
            )}
            <Button size="sm" disabled={completing || completeRequested} onClick={handleComplete}>
              完了にする
            </Button>
          </>
        )}
      </div>
    </div>
  )
}

export { TrialHeader }
export type { TrialHeaderProps }
