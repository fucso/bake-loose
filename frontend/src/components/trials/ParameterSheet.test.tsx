import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Provider, type Client } from 'urql'
import { describe, expect, it, vi } from 'vitest'

import { ParameterSheet } from './ParameterSheet'
import {
  createMockClient,
  createRecordingClient,
  MockGraphQLError,
  type RecordedOperation,
} from '../../../test/mocks/urql'
import type { Parameter } from '@/lib/trial'

const renderSheet = (client: Client, parameter?: Parameter) => {
  const onOpenChange = vi.fn()
  const onSaved = vi.fn()
  const utils = render(
    <Provider value={client}>
      <ParameterSheet
        open
        onOpenChange={onOpenChange}
        trialId="trial-1"
        stepId="step-1"
        parameter={parameter}
        onSaved={onSaved}
      />
    </Provider>,
  )
  return { ...utils, onOpenChange, onSaved }
}

const addResponse = { AddParameter: { addParameter: { id: 'new-param' } } }
const updateResponse = { UpdateParameter: { updateParameter: { id: 'param-1' } } }

const lastVariables = (operations: RecordedOperation[]) =>
  operations[operations.length - 1].variables

const selectType = (label: string) => {
  fireEvent.click(screen.getByRole('radio', { name: label }))
}

const save = () => {
  fireEvent.click(screen.getByRole('button', { name: '保存' }))
}

describe('ParameterSheet', () => {
  describe('追加', () => {
    it('初期表示は項目と値のフォームで、種別を選ぶとフォームが切り替わる', () => {
      renderSheet(createMockClient({}))

      expect(screen.getByLabelText('項目名')).toBeInTheDocument()

      selectType('経過時間')

      expect(screen.queryByLabelText('項目名')).not.toBeInTheDocument()
      expect(screen.getByLabelText('時間')).toBeInTheDocument()
    })

    it('数量の項目と値を追加する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      fireEvent.change(screen.getByLabelText('項目名'), { target: { value: '強力粉' } })
      fireEvent.change(screen.getByLabelText('数量'), { target: { value: '300' } })
      fireEvent.change(screen.getByLabelText('単位'), { target: { value: 'g' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations)).toEqual({
        trialId: 'trial-1',
        stepId: 'step-1',
        content: {
          type: 'key_value',
          key: '強力粉',
          value: { type: 'quantity', amount: 300, unit: 'g' },
        },
      })
    })

    it('値の種類を文字列に切り替えて追加する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      fireEvent.click(screen.getByRole('radio', { name: '文字列' }))
      fireEvent.change(screen.getByLabelText('項目名'), { target: { value: '発酵場所' } })
      fireEvent.change(screen.getByLabelText('値'), { target: { value: '冷蔵庫' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations).content).toEqual({
        type: 'key_value',
        key: '発酵場所',
        value: { type: 'text', value: '冷蔵庫' },
      })
    })

    it('経過時間を単位付きで追加する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      selectType('経過時間')
      fireEvent.change(screen.getByLabelText('内容'), { target: { value: '一次発酵' } })
      fireEvent.change(screen.getByLabelText('時間'), { target: { value: '2' } })
      fireEvent.change(screen.getByLabelText('単位'), { target: { value: 'hour' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations).content).toEqual({
        type: 'duration',
        duration: { value: 2, unit: 'hour' },
        note: '一次発酵',
      })
    })

    it('時間マーカーを追加する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      selectType('時間マーカー')
      fireEvent.change(screen.getByLabelText('内容'), { target: { value: '焼成開始から' } })
      fireEvent.change(screen.getByLabelText('経過時点'), { target: { value: '30' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations).content).toEqual({
        type: 'time_marker',
        at: { value: 30, unit: 'minute' },
        note: '焼成開始から',
      })
    })

    it('自由記述を追加する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      selectType('自由記述')
      fireEvent.change(screen.getByLabelText('内容'), { target: { value: '打ち粉を追加' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations).content).toEqual({
        type: 'text',
        value: '打ち粉を追加',
      })
    })

    it.each([
      ['経過時間', '時間'],
      ['時間マーカー', '経過時点'],
    ])('%sは内容が未入力だと送信せずエラーを表示する', async (typeLabel, timeLabel) => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      selectType(typeLabel)
      fireEvent.change(screen.getByLabelText(timeLabel), { target: { value: '90' } })
      save()

      expect(await screen.findByText('内容を入力してください')).toBeInTheDocument()
      expect(operations).toHaveLength(0)
      expect(onSaved).not.toHaveBeenCalled()
    })

    it('入力に不備がある場合は送信せずエラーを表示する', async () => {
      const { client, operations } = createRecordingClient(addResponse)
      const { onSaved } = renderSheet(client)

      save()

      expect(await screen.findByText('項目名を入力してください')).toBeInTheDocument()
      expect(operations).toHaveLength(0)
      expect(onSaved).not.toHaveBeenCalled()
    })

    it('入力を直すと検証エラーの表示を消す', async () => {
      renderSheet(createMockClient({}))

      save()
      expect(await screen.findByText('項目名を入力してください')).toBeInTheDocument()

      fireEvent.change(screen.getByLabelText('項目名'), { target: { value: '強力粉' } })

      expect(screen.queryByText('項目名を入力してください')).not.toBeInTheDocument()
    })

    it('追加に失敗した場合はバックエンドのエラーメッセージを表示しシートを閉じない', async () => {
      const { onSaved, onOpenChange } = renderSheet(
        createMockClient({ AddParameter: new MockGraphQLError('工程は完了済みです') }),
      )

      fireEvent.change(screen.getByLabelText('項目名'), { target: { value: '強力粉' } })
      fireEvent.change(screen.getByLabelText('数量'), { target: { value: '300' } })
      fireEvent.change(screen.getByLabelText('単位'), { target: { value: 'g' } })
      save()

      expect(await screen.findByText('工程は完了済みです')).toBeInTheDocument()
      expect(onSaved).not.toHaveBeenCalled()
      expect(onOpenChange).not.toHaveBeenCalledWith(false)
    })
  })

  describe('編集', () => {
    const durationParameter: Parameter = {
      id: 'param-1',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
    }

    const quantityParameter: Parameter = {
      id: 'param-2',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    }

    it('既存の値を初期表示し、種別は選択できない', () => {
      renderSheet(createMockClient({}), durationParameter)

      expect(screen.getByLabelText('内容')).toHaveValue('一次発酵')
      expect(screen.getByLabelText('時間')).toHaveValue('90')
      expect(screen.queryByRole('radio', { name: '経過時間' })).not.toBeInTheDocument()
      expect(screen.getByText('種別: 経過時間（変更できません）')).toBeInTheDocument()
    })

    it('KeyValueの値の種類は変更できない', () => {
      renderSheet(createMockClient({}), quantityParameter)

      expect(screen.getByRole('radio', { name: '数値' })).toBeDisabled()
      expect(screen.getByRole('radio', { name: '文字列' })).toBeDisabled()
    })

    it('自由記述の値を更新する', async () => {
      const { client, operations } = createRecordingClient(updateResponse)
      const { onSaved } = renderSheet(client, {
        id: 'param-3',
        parameterType: 'TEXT',
        content: { type: 'text', value: '打ち粉を追加' },
      })

      expect(screen.getByLabelText('内容')).toHaveValue('打ち粉を追加')
      fireEvent.change(screen.getByLabelText('内容'), { target: { value: '打ち粉を多めに' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations).content).toEqual({
        type: 'text',
        value: '打ち粉を多めに',
      })
    })

    it('更新に失敗した場合はバックエンドのエラーメッセージを表示しシートを閉じない', async () => {
      const { onSaved, onOpenChange } = renderSheet(
        createMockClient({ UpdateParameter: new MockGraphQLError('試行は完了済みです') }),
        durationParameter,
      )

      fireEvent.change(screen.getByLabelText('時間'), { target: { value: '120' } })
      save()

      expect(await screen.findByText('試行は完了済みです')).toBeInTheDocument()
      expect(onSaved).not.toHaveBeenCalled()
      expect(onOpenChange).not.toHaveBeenCalledWith(false)
    })

    it('末端の値を更新する', async () => {
      const { client, operations } = createRecordingClient(updateResponse)
      const { onSaved, onOpenChange } = renderSheet(client, durationParameter)

      fireEvent.change(screen.getByLabelText('時間'), { target: { value: '120' } })
      save()

      await waitFor(() => {
        expect(onSaved).toHaveBeenCalledTimes(1)
      })
      expect(lastVariables(operations)).toEqual({
        trialId: 'trial-1',
        stepId: 'step-1',
        parameterId: 'param-1',
        content: {
          type: 'duration',
          duration: { value: 120, unit: 'minute' },
          note: '一次発酵',
        },
      })
      expect(onOpenChange).toHaveBeenCalledWith(false)
    })
  })
})
