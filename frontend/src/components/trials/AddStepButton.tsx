import { useState } from "react"
import { useMutation } from "urql"

import { StepFormModal, type StepFormValues } from "@/components/trials/StepFormModal"
import { Button } from "@/components/ui/button"
import { toUserFacingErrorMessage } from "@/lib/graphql-error"

type AddStepData = {
  addStep: {
    id: string
    name: string
    position: number
    startedAt: string | null
  }
}

type AddStepVariables = {
  trialId: string
  input: {
    name: string
    startedAt: string | null
  }
}

// position は追加時に既存の工程数から自動採番されるため、入力には含めない
const ADD_STEP_MUTATION = `
  mutation AddStep($trialId: ID!, $input: AddStepInput!) {
    addStep(trialId: $trialId, input: $input) {
      id
      name
      position
      startedAt
    }
  }
`

type AddStepButtonProps = {
  /** 工程を追加する対象の Trial ID */
  trialId: string
  /** 工程の追加に成功したときのハンドラ */
  onAdded: () => void
}

/**
 * タイムライン末尾に置く工程の追加操作。
 *
 * 追加された工程は position の末尾に積まれるため、成功時は再取得に任せて
 * タイムラインの末尾へ反映する（このコンポーネントは表示順を持たない）。
 */
const AddStepButton = ({ trialId, onAdded }: AddStepButtonProps) => {
  const [isOpen, setIsOpen] = useState(false)
  // ミューテーションの error は次の実行までクリアされないため、表示用のメッセージは
  // 自前で保持する。開き直したときに前回の失敗を持ち越さないようにするため。
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [{ fetching }, addStep] = useMutation<AddStepData, AddStepVariables>(ADD_STEP_MUTATION)

  const handleOpenChange = (nextOpen: boolean) => {
    if (nextOpen) {
      setErrorMessage(null)
    }
    setIsOpen(nextOpen)
  }

  const handleSubmit = async ({ name, startedAt }: StepFormValues) => {
    const result = await addStep({ trialId, input: { name, startedAt } })
    if (result.error) {
      setErrorMessage(toUserFacingErrorMessage(result.error, "工程の追加に失敗しました"))
      return
    }

    setIsOpen(false)
    onAdded()
  }

  return (
    <>
      {/* 追加操作であることが一目で分かるよう、既存の新規作成ボタンと同じく先頭に + を置く */}
      <Button variant="outline" size="sm" onClick={() => handleOpenChange(true)}>
        + 工程を追加
      </Button>

      <StepFormModal
        open={isOpen}
        onOpenChange={handleOpenChange}
        title="工程を追加"
        submitLabel="追加"
        initialName=""
        initialStartedAt={null}
        submitting={fetching}
        errorMessage={errorMessage}
        onSubmit={handleSubmit}
      />
    </>
  )
}

export { AddStepButton }
export type { AddStepButtonProps }
