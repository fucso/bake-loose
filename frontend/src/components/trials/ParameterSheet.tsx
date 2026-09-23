import { useState, type FormEvent, type ReactNode } from "react"
import { useMutation } from "urql"

import { Button, buttonVariants } from "@/components/ui/button"
import { Dialog, DialogClose, DialogSheetPopup, DialogTitle } from "@/components/ui/dialog"
import { Input, inputClassName } from "@/components/ui/input"
import { toUserFacingErrorMessage } from "@/lib/graphql-error"
import {
  buildParameterContent,
  createEmptyFormState,
  DURATION_UNITS,
  PARAMETER_TYPES,
  toFormState,
  type ParameterFormState,
} from "@/lib/parameter-content"
import { DURATION_UNIT_LABELS, PARAMETER_TYPE_LABELS } from "@/lib/parameter-format"
import type { Parameter, ParameterContent } from "@/lib/trial"
import { cn } from "@/lib/utils"

type ParameterMutationResult = {
  id: string
}

type AddParameterData = {
  addParameter: ParameterMutationResult
}

type AddParameterVariables = {
  trialId: string
  stepId: string
  content: ParameterContent
}

// 反映は Trial の再取得で行うため、レスポンスは成功可否が分かる最小限にとどめる
const ADD_PARAMETER_MUTATION = `
  mutation AddParameter($trialId: ID!, $stepId: ID!, $content: JSON!) {
    addParameter(trialId: $trialId, stepId: $stepId, content: $content) {
      id
    }
  }
`

type UpdateParameterData = {
  updateParameter: ParameterMutationResult
}

type UpdateParameterVariables = AddParameterVariables & {
  parameterId: string
}

const UPDATE_PARAMETER_MUTATION = `
  mutation UpdateParameter($trialId: ID!, $stepId: ID!, $parameterId: ID!, $content: JSON!) {
    updateParameter(
      trialId: $trialId
      stepId: $stepId
      parameterId: $parameterId
      content: $content
    ) {
      id
    }
  }
`

/** 選択肢をボタン状に並べるためのラベル。入力自体は視覚的に隠したラジオが担う */
const choiceLabelClassName =
  "flex cursor-pointer items-center justify-center rounded-md border border-input px-3 py-2 text-sm select-none has-[:checked]:border-primary has-[:checked]:bg-primary/10 has-[:disabled]:cursor-default has-[:disabled]:opacity-50 has-[:focus-visible]:border-ring has-[:focus-visible]:ring-3 has-[:focus-visible]:ring-ring/50"

type ChoiceProps = {
  name: string
  checked: boolean
  disabled?: boolean
  onSelect: () => void
  children: ReactNode
}

/** ボタン状に見せるラジオ。調理中でも押しやすいよう、プルダウンを開かずに選べるようにする */
const Choice = ({ name, checked, disabled, onSelect, children }: ChoiceProps) => {
  return (
    <label className={choiceLabelClassName}>
      <input
        type="radio"
        name={name}
        checked={checked}
        disabled={disabled}
        onChange={onSelect}
        className="sr-only"
      />
      {children}
    </label>
  )
}

type FieldProps = {
  label: string
  children: ReactNode
}

const Field = ({ label, children }: FieldProps) => {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span>{label}</span>
      {children}
    </label>
  )
}

type ParameterSheetProps = {
  /** シートの開閉状態 */
  open: boolean
  /** 開閉状態が変化したときのハンドラ（背景クリック・Escキー押下時にも呼ばれる） */
  onOpenChange: (open: boolean) => void
  /** 対象の Trial ID */
  trialId: string
  /** 対象の Step ID */
  stepId: string
  /** 編集対象のパラメーター。省略した場合は新規追加として扱う */
  parameter?: Parameter
  /** 追加・更新に成功したときのハンドラ */
  onSaved: () => void
}

/**
 * パラメーターの追加・編集を行うボトムシート。
 *
 * 調理中の片手操作を前提に、画面下端から開くシートに入力を集約する。
 * 追加時は種別を選んでから種別ごとのフォームを出す。編集時は種別を変更できない
 * （バックエンドが `ParameterContent` のバリアント変更を拒否するため）ので種別選択は出さず、
 * KeyValue の値の種類（文字列 / 数量）も固定する。
 *
 * フォームの初期状態は生成時に確定させるため、呼び出し側はシートを開くときにマウントし、
 * 閉じるときにアンマウントすること。
 */
const ParameterSheet = ({
  open,
  onOpenChange,
  trialId,
  stepId,
  parameter,
  onSaved,
}: ParameterSheetProps) => {
  const isEditing = parameter !== undefined
  const [form, setForm] = useState<ParameterFormState>(() =>
    parameter ? toFormState(parameter) : createEmptyFormState()
  )
  const [validationError, setValidationError] = useState<string | null>(null)

  const [{ fetching: adding, error: addError }, addParameter] = useMutation<
    AddParameterData,
    AddParameterVariables
  >(ADD_PARAMETER_MUTATION)
  const [{ fetching: updating, error: updateError }, updateParameter] = useMutation<
    UpdateParameterData,
    UpdateParameterVariables
  >(UPDATE_PARAMETER_MUTATION)

  const saving = adding || updating

  const updateForm = <K extends keyof ParameterFormState>(
    key: K,
    value: ParameterFormState[K]
  ) => {
    setForm((current) => ({ ...current, [key]: value }))
    // 指摘された項目を直したのにメッセージが残ると、何が未解決か分からなくなるため入力時に消す
    setValidationError(null)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    const built = buildParameterContent(form)
    if (!built.ok) {
      setValidationError(built.message)
      return
    }
    setValidationError(null)

    const result = parameter
      ? await updateParameter({
          trialId,
          stepId,
          parameterId: parameter.id,
          content: built.content,
        })
      : await addParameter({ trialId, stepId, content: built.content })

    if (!result.error) {
      onSaved()
      onOpenChange(false)
    }
  }

  const errorMessage =
    validationError ??
    toUserFacingErrorMessage(
      isEditing ? updateError : addError,
      isEditing ? "パラメーターの更新に失敗しました" : "パラメーターの追加に失敗しました"
    )

  const timeFieldLabel = form.parameterType === "DURATION" ? "時間" : "経過時点"

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogSheetPopup>
        <DialogTitle>
          {isEditing ? "パラメーターを編集" : "パラメーターを追加"}
        </DialogTitle>

        <form onSubmit={handleSubmit} className="flex flex-col gap-3">
          {isEditing ? (
            <p className="text-sm text-muted-foreground">
              種別: {PARAMETER_TYPE_LABELS[form.parameterType]}（変更できません）
            </p>
          ) : (
            <fieldset className="flex flex-col gap-1 text-sm">
              <legend className="mb-1">種別</legend>
              <div className="grid grid-cols-2 gap-2">
                {PARAMETER_TYPES.map((type) => (
                  <Choice
                    key={type}
                    name="parameterType"
                    checked={form.parameterType === type}
                    onSelect={() => updateForm("parameterType", type)}
                  >
                    {PARAMETER_TYPE_LABELS[type]}
                  </Choice>
                ))}
              </div>
            </fieldset>
          )}

          {form.parameterType === "KEY_VALUE" && (
            <>
              <Field label="項目名">
                <Input
                  autoFocus
                  value={form.key}
                  onChange={(event) => updateForm("key", event.target.value)}
                  disabled={saving}
                />
              </Field>

              <fieldset className="flex flex-col gap-1 text-sm">
                <legend className="mb-1">値の種類</legend>
                <div className="grid grid-cols-2 gap-2">
                  <Choice
                    name="valueKind"
                    checked={form.valueKind === "quantity"}
                    disabled={isEditing || saving}
                    onSelect={() => updateForm("valueKind", "quantity")}
                  >
                    数値
                  </Choice>
                  <Choice
                    name="valueKind"
                    checked={form.valueKind === "text"}
                    disabled={isEditing || saving}
                    onSelect={() => updateForm("valueKind", "text")}
                  >
                    文字列
                  </Choice>
                </div>
              </fieldset>

              {form.valueKind === "quantity" ? (
                <div className="grid grid-cols-2 gap-2">
                  <Field label="数量">
                    <Input
                      inputMode="decimal"
                      value={form.amount}
                      onChange={(event) => updateForm("amount", event.target.value)}
                      disabled={saving}
                    />
                  </Field>
                  <Field label="単位">
                    <Input
                      value={form.unit}
                      onChange={(event) => updateForm("unit", event.target.value)}
                      disabled={saving}
                    />
                  </Field>
                </div>
              ) : (
                <Field label="値">
                  <Input
                    value={form.keyValueText}
                    onChange={(event) => updateForm("keyValueText", event.target.value)}
                    disabled={saving}
                  />
                </Field>
              )}
            </>
          )}

          {(form.parameterType === "DURATION" || form.parameterType === "TIME_MARKER") && (
            <>
              <Field label="内容">
                <Input
                  autoFocus
                  value={form.note}
                  onChange={(event) => updateForm("note", event.target.value)}
                  disabled={saving}
                />
              </Field>

              <div className="grid grid-cols-2 gap-2">
                <Field label={timeFieldLabel}>
                  <Input
                    inputMode="decimal"
                    value={form.time}
                    onChange={(event) => updateForm("time", event.target.value)}
                    disabled={saving}
                  />
                </Field>
                <Field label="単位">
                  <select
                    value={form.timeUnit}
                    onChange={(event) =>
                      updateForm("timeUnit", event.target.value as ParameterFormState["timeUnit"])
                    }
                    disabled={saving}
                    className={inputClassName}
                  >
                    {DURATION_UNITS.map((unit) => (
                      <option key={unit} value={unit}>
                        {DURATION_UNIT_LABELS[unit]}
                      </option>
                    ))}
                  </select>
                </Field>
              </div>
            </>
          )}

          {form.parameterType === "TEXT" && (
            <Field label="内容">
              <textarea
                autoFocus
                rows={3}
                value={form.text}
                onChange={(event) => updateForm("text", event.target.value)}
                disabled={saving}
                className={inputClassName}
              />
            </Field>
          )}

          {errorMessage && <p className="text-sm text-destructive">{errorMessage}</p>}

          <div className="mt-2 flex justify-end gap-2">
            <DialogClose
              type="button"
              disabled={saving}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              キャンセル
            </DialogClose>
            <Button type="submit" disabled={saving}>
              保存
            </Button>
          </div>
        </form>
      </DialogSheetPopup>
    </Dialog>
  )
}

export { ParameterSheet }
export type { ParameterSheetProps }
