import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Provider, type Client } from 'urql'
import { describe, expect, it, vi } from 'vitest'

import { StepParameterList } from './StepParameterList'
import { createMockClient, createRecordingClient, MockGraphQLError } from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const step: Step = {
  id: 'step-1',
  name: '一次発酵',
  position: 0,
  startedAt: null,
  completedAt: null,
  isCompleted: false,
  parameters: [
    {
      id: 'param-1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    },
    {
      id: 'param-2',
      parameterType: 'TEXT',
      content: { type: 'text', value: '打ち粉を追加' },
    },
  ],
}

const renderList = (client: Client, { editable = true, target = step } = {}) => {
  const onChanged = vi.fn()
  const utils = render(
    <Provider value={client}>
      <StepParameterList trialId="trial-1" step={target} editable={editable} onChanged={onChanged} />
    </Provider>,
  )
  return { ...utils, onChanged }
}

describe('StepParameterList', () => {
  it('パラメーターが無い場合は未記録であることを表示する', () => {
    renderList(createMockClient({}), { target: { ...step, parameters: [] } })

    expect(screen.getByText('パラメーターは記録されていません')).toBeInTheDocument()
  })

  it('記録できない場合は操作 UI を出さない', () => {
    renderList(createMockClient({}), { editable: false })

    expect(screen.getByText('300g')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'パラメーター追加' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '強力粉 を編集' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '強力粉 を削除' })).not.toBeInTheDocument()
  })

  it('ラベルを持たないパラメーターは値で操作対象を示す', () => {
    renderList(createMockClient({}))

    expect(screen.getByRole('button', { name: '打ち粉を追加 を編集' })).toBeInTheDocument()
  })

  it('追加操作でシートが開く', () => {
    renderList(createMockClient({}))

    fireEvent.click(screen.getByRole('button', { name: 'パラメーター追加' }))

    expect(screen.getByText('パラメーターを追加')).toBeInTheDocument()
  })

  it('編集操作で対象パラメーターの値を入れたシートが開く', () => {
    renderList(createMockClient({}))

    fireEvent.click(screen.getByRole('button', { name: '強力粉 を編集' }))

    expect(screen.getByText('パラメーターを編集')).toBeInTheDocument()
    expect(screen.getByLabelText('項目名')).toHaveValue('強力粉')
    expect(screen.getByLabelText('数量')).toHaveValue('300')
  })

  it('削除は確認してから実行する', async () => {
    const { client, operations } = createRecordingClient({
      RemoveParameter: { removeParameter: { id: 'step-1' } },
    })
    const { onChanged } = renderList(client)

    fireEvent.click(screen.getByRole('button', { name: '強力粉 を削除' }))

    // 確認中はまだ削除しない
    expect(operations).toHaveLength(0)
    expect(screen.getByText('削除しますか？')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '削除' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
    expect(operations[0]).toEqual({
      name: 'RemoveParameter',
      variables: { trialId: 'trial-1', stepId: 'step-1', parameterId: 'param-1' },
    })
  })

  it('未知の種別は編集させず削除だけ行える', () => {
    const unknownParameter = {
      id: 'param-9',
      parameterType: 'COLOR',
      content: { type: 'color', value: 'きつね色' },
    } as unknown as Step['parameters'][number]

    renderList(createMockClient({}), {
      target: { ...step, parameters: [unknownParameter] },
    })

    const editButtons = screen.queryAllByRole('button', { name: /を編集$/ })
    expect(editButtons).toHaveLength(0)
    expect(screen.getAllByRole('button', { name: /を削除$/ })).toHaveLength(1)
  })

  it('削除の確認をやめると削除しない', () => {
    const { client, operations } = createRecordingClient({})
    const { onChanged } = renderList(client)

    fireEvent.click(screen.getByRole('button', { name: '強力粉 を削除' }))
    fireEvent.click(screen.getByRole('button', { name: 'やめる' }))

    expect(screen.queryByText('削除しますか？')).not.toBeInTheDocument()
    expect(operations).toHaveLength(0)
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('削除に失敗した場合はエラーメッセージを表示し、確認をやめると消す', async () => {
    const { onChanged } = renderList(
      createMockClient({ RemoveParameter: new MockGraphQLError('試行は完了済みです') }),
    )

    fireEvent.click(screen.getByRole('button', { name: '強力粉 を削除' }))
    fireEvent.click(screen.getByRole('button', { name: '削除' }))

    expect(await screen.findByText('試行は完了済みです')).toBeInTheDocument()
    expect(onChanged).not.toHaveBeenCalled()

    fireEvent.click(screen.getByRole('button', { name: 'やめる' }))

    expect(screen.queryByText('試行は完了済みです')).not.toBeInTheDocument()
  })
})
