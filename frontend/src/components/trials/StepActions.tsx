import { useState } from "react"
import { useMutation } from "urql"

import { StepFormModal, type StepFormValues } from "@/components/trials/StepFormModal"
import { Button } from "@/components/ui/button"
import { toUserFacingErrorMessage } from "@/lib/graphql-error"
import type { Step } from "@/lib/trial"

type UpdateStepData = {
  updateStep: {
    id: string
    name: string
    startedAt: string | null
  }
}

type UpdateStepVariables = {
  trialId: string
  stepId: string
  input: {
    name: string
    startedAt: string | null
  }
}

const UPDATE_STEP_MUTATION = `
  mutation UpdateStep($trialId: ID!, $stepId: ID!, $input: UpdateStepInput!) {
    updateStep(trialId: $trialId, stepId: $stepId, input: $input) {
      id
      name
      startedAt
    }
  }
`

type CompleteStepData = {
  completeStep: {
    id: string
    isCompleted: boolean
    completedAt: string | null
  }
}

type CompleteStepVariables = {
  trialId: string
  stepId: string
}

// completedAt を省略するとバックエンドが現在時刻を設定する
const COMPLETE_STEP_MUTATION = `
  mutation CompleteStep($trialId: ID!, $stepId: ID!) {
    completeStep(trialId: $trialId, stepId: $stepId) {
      id
      isCompleted
      completedAt
    }
  }
`

type StepActionsProps = {
  /** 工程が属する Trial の ID */
  trialId: string
  /** 操作対象の工程 */
  step: Step
  /** 工程の更新・完了に成功したときのハンドラ */
  onChanged: () => void
}

/**
 * 工程1件に対する記録操作（編集・完了）。
 *
 * 記録中はカードを開いたまま操作できるよう、工程カードの本文内に配置する。
 * 完了済みの工程・完了済み Trial では操作できないため、表示するかどうかは呼び出し元が判断する。
 */
const StepActions = ({ trialId, step, onChanged }: StepActionsProps) => {
  const [isEditing, setIsEditing] = useState(false)
  // ミューテーションの error は次の実行までクリアされないため、表示用のメッセージは
  // 自前で保持し、次の操作を始めた時点で破棄する
  const [updateErrorMessage, setUpdateErrorMessage] = useState<string | null>(null)
  const [completeErrorMessage, setCompleteErrorMessage] = useState<string | null>(null)

  const [{ fetching: updating }, updateStep] = useMutation<UpdateStepData, UpdateStepVariables>(
    UPDATE_STEP_MUTATION
  )
  const [{ fetching: completing }, completeStep] = useMutation<
    CompleteStepData,
    CompleteStepVariables
  >(COMPLETE_STEP_MUTATION)

  // 片方の送信中にもう片方を実行すると2つの更新が並走し、どちらの結果が
  // 最後に反映されるかが不定になるため、どちらか実行中は両方を止める
  const isSubmitting = updating || completing

  const handleOpenChange = (nextOpen: boolean) => {
    if (nextOpen) {
      setUpdateErrorMessage(null)
    }
    setIsEditing(nextOpen)
  }

  const handleSubmit = async ({ name, startedAt }: StepFormValues) => {
    // フォームは常に現在値を表示するため name / startedAt をともに送る。
    // startedAt の空欄は「未設定に戻す」意図として null を送り、バックエンド側の値をクリアする。
    const result = await updateStep({ trialId, stepId: step.id, input: { name, startedAt } })
    if (result.error) {
      setUpdateErrorMessage(toUserFacingErrorMessage(result.error, "工程の更新に失敗しました"))
      return
    }

    setIsEditing(false)
    setCompleteErrorMessage(null)
    onChanged()
  }

  const handleComplete = async () => {
    setCompleteErrorMessage(null)

    const result = await completeStep({ trialId, stepId: step.id })
    if (result.error) {
      setCompleteErrorMessage(toUserFacingErrorMessage(result.error, "工程の完了に失敗しました"))
      return
    }

    onChanged()
  }

  return (
    <div data-slot="step-actions" className="flex flex-col gap-2">
      {completeErrorMessage && (
        <p role="alert" className="text-sm text-destructive">
          {completeErrorMessage}
        </p>
      )}

      <div className="flex flex-wrap gap-2">
        <Button
          variant="outline"
          size="sm"
          disabled={isSubmitting}
          onClick={() => handleOpenChange(true)}
        >
          工程を編集
        </Button>
        <Button size="sm" disabled={isSubmitting} onClick={handleComplete}>
          工程を完了にする
        </Button>
      </div>

      <StepFormModal
        open={isEditing}
        onOpenChange={handleOpenChange}
        title="工程を編集"
        submitLabel="保存"
        initialName={step.name}
        initialStartedAt={step.startedAt}
        submitting={updating}
        errorMessage={updateErrorMessage}
        onSubmit={handleSubmit}
      />
    </div>
  )
}

export { StepActions }
export type { StepActionsProps }
