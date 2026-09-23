import { Pencil, Plus, Trash2 } from "lucide-react"
import { useState } from "react"
import { useMutation } from "urql"

import { ParameterItem } from "@/components/trials/ParameterItem"
import { ParameterSheet } from "@/components/trials/ParameterSheet"
import { Button } from "@/components/ui/button"
import { toUserFacingErrorMessage } from "@/lib/graphql-error"
import { isEditableParameter } from "@/lib/parameter-content"
import { formatParameter } from "@/lib/parameter-format"
import type { Parameter, Step } from "@/lib/trial"

type RemoveParameterData = {
  /** `removeParameter` が返すのは削除後の Step */
  removeParameter: {
    id: string
  }
}

type RemoveParameterVariables = {
  trialId: string
  stepId: string
  parameterId: string
}

const REMOVE_PARAMETER_MUTATION = `
  mutation RemoveParameter($trialId: ID!, $stepId: ID!, $parameterId: ID!) {
    removeParameter(trialId: $trialId, stepId: $stepId, parameterId: $parameterId) {
      id
    }
  }
`

/** 入力シートの対象。編集は対象パラメーターを伴う */
type SheetTarget = { mode: "add" } | { mode: "edit"; parameter: Parameter }

/** 操作ボタンの表記に使う、パラメーターを一意に指す呼び名 */
const parameterName = (parameter: Parameter): string => {
  const { label, value } = formatParameter(parameter)
  return label ?? value
}

type StepParameterListProps = {
  /** 対象の Trial ID */
  trialId: string
  /** 表示する工程 */
  step: Step
  /**
   * パラメーターを記録できるか。
   * バックエンドは完了済みの Trial・工程へのパラメーター操作を拒否するため、
   * 記録できない場合は操作 UI 自体を出さない。
   */
  editable: boolean
  /** 追加・編集・削除に成功したときのハンドラ */
  onChanged: () => void
}

/**
 * 工程1件に紐づくパラメーターの一覧表示と記録操作（追加・編集・削除）を担う。
 *
 * 記録操作は工程カードの中で完結させ、画面遷移なしに記録を続けられるようにする。
 * 削除は取り消せないため、同じ行でもう一度確認してから実行する。
 */
const StepParameterList = ({ trialId, step, editable, onChanged }: StepParameterListProps) => {
  const [sheetTarget, setSheetTarget] = useState<SheetTarget | null>(null)
  const [confirmingId, setConfirmingId] = useState<string | null>(null)

  const [{ fetching: removing, error: removeError }, removeParameter] = useMutation<
    RemoveParameterData,
    RemoveParameterVariables
  >(REMOVE_PARAMETER_MUTATION)

  const handleRemove = async (parameterId: string) => {
    const result = await removeParameter({ trialId, stepId: step.id, parameterId })
    if (!result.error) {
      setConfirmingId(null)
      onChanged()
    }
  }

  const removeErrorMessage = toUserFacingErrorMessage(
    removeError,
    "パラメーターの削除に失敗しました"
  )

  const renderActions = (parameter: Parameter) => {
    if (!editable) {
      return undefined
    }

    const name = parameterName(parameter)

    if (confirmingId === parameter.id) {
      return (
        <span className="flex shrink-0 items-center gap-1">
          <span className="text-xs text-muted-foreground">削除しますか？</span>
          <Button
            type="button"
            size="xs"
            variant="destructive"
            disabled={removing}
            onClick={() => handleRemove(parameter.id)}
          >
            削除
          </Button>
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={removing}
            onClick={() => setConfirmingId(null)}
          >
            やめる
          </Button>
        </span>
      )
    }

    return (
      <span className="flex shrink-0 items-center gap-1">
        {/* 未知の種別は入力フォームに写せないため編集させない（削除は種別によらず行える） */}
        {isEditableParameter(parameter) && (
          <Button
            type="button"
            size="icon-xs"
            variant="ghost"
            aria-label={`${name} を編集`}
            onClick={() => setSheetTarget({ mode: "edit", parameter })}
          >
            <Pencil aria-hidden="true" />
          </Button>
        )}
        <Button
          type="button"
          size="icon-xs"
          variant="ghost"
          aria-label={`${name} を削除`}
          onClick={() => setConfirmingId(parameter.id)}
        >
          <Trash2 aria-hidden="true" />
        </Button>
      </span>
    )
  }

  return (
    <div className="flex flex-col gap-2">
      {step.parameters.length === 0 ? (
        <p className="text-sm text-muted-foreground">パラメーターは記録されていません</p>
      ) : (
        <ul>
          {step.parameters.map((parameter) => (
            <ParameterItem
              key={parameter.id}
              parameter={parameter}
              actions={renderActions(parameter)}
            />
          ))}
        </ul>
      )}

      {/* 削除の確認をやめた時点で用済みになるため、確認中の間だけ出す */}
      {confirmingId !== null && removeErrorMessage && (
        <p className="text-sm text-destructive">{removeErrorMessage}</p>
      )}

      {editable && (
        <div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() => setSheetTarget({ mode: "add" })}
          >
            <Plus aria-hidden="true" data-icon="inline-start" />
            パラメーター追加
          </Button>
        </div>
      )}

      {/*
        シートは初期値を生成時に確定させるため、開いている間だけマウントする。
        対象を切り替えても前のパラメーターの入力内容が残らない。
      */}
      {sheetTarget && (
        <ParameterSheet
          open
          onOpenChange={(open) => {
            if (!open) {
              setSheetTarget(null)
            }
          }}
          trialId={trialId}
          stepId={step.id}
          parameter={sheetTarget.mode === "edit" ? sheetTarget.parameter : undefined}
          onSaved={onChanged}
        />
      )}
    </div>
  )
}

export { StepParameterList }
export type { StepParameterListProps }
