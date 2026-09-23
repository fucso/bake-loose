import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Provider } from 'urql'
import { describe, expect, it, vi } from 'vitest'

import { StepActions } from './StepActions'
import {
  createMockClient,
  MockGraphQLError,
  type MockQueryResponses,
} from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const buildStep = (overrides: Partial<Step> = {}): Step => ({
  id: 'step-1',
  name: 'こね',
  position: 0,
  startedAt: '2026-01-01T09:00:00+09:00',
  completedAt: null,
  isCompleted: false,
  parameters: [],
  ...overrides,
})

const renderActions = (
  client: ReturnType<typeof createMockClient>,
  step: Step = buildStep(),
) => {
  const onChanged = vi.fn()
  const utils = render(
    <Provider value={client}>
      <StepActions trialId="trial-1" step={step} onChanged={onChanged} />
    </Provider>,
  )
  return { ...utils, onChanged }
}

describe('StepActions', () => {
  it('編集操作を開くと工程の現在値を入力欄に表示する', () => {
    renderActions(createMockClient({}))

    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))

    expect(screen.getByLabelText('工程名')).toHaveValue('こね')
    expect(screen.getByLabelText('開始日時')).toHaveValue('2026-01-01T09:00')
  })

  it('編集して保存するとonChangedが呼ばれフォームが閉じる', async () => {
    const client = createMockClient({
      UpdateStep: {
        updateStep: { id: 'step-1', name: 'こね直し', startedAt: '2026-01-01T09:00:00+09:00' },
      },
    })
    const { onChanged } = renderActions(client)

    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))
    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: 'こね直し' } })
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
    expect(screen.queryByLabelText('工程名')).not.toBeInTheDocument()
  })

  it('編集に失敗した場合はエラーメッセージを表示しonChangedは呼ばれない', async () => {
    const client = createMockClient({
      UpdateStep: new MockGraphQLError('工程名が長すぎます'),
    })
    const { onChanged } = renderActions(client)

    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    expect(await screen.findByText('工程名が長すぎます')).toBeInTheDocument()
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('完了操作でcompleteStepが成功するとonChangedが呼ばれる', async () => {
    const client = createMockClient({
      CompleteStep: {
        completeStep: {
          id: 'step-1',
          isCompleted: true,
          completedAt: '2026-01-01T09:30:00+09:00',
        },
      },
    })
    const { onChanged } = renderActions(client)

    fireEvent.click(screen.getByRole('button', { name: '工程を完了にする' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
  })

  it('完了に失敗した場合はエラーメッセージを表示しonChangedは呼ばれない', async () => {
    const client = createMockClient({
      CompleteStep: new MockGraphQLError('既に完了している工程です'),
    })
    const { onChanged } = renderActions(client)

    fireEvent.click(screen.getByRole('button', { name: '工程を完了にする' }))

    expect(await screen.findByText('既に完了している工程です')).toBeInTheDocument()
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('失敗した編集フォームを閉じて開き直すと前回のエラーメッセージを持ち越さない', async () => {
    const client = createMockClient({
      UpdateStep: new MockGraphQLError('工程名が長すぎます'),
    })
    renderActions(client)

    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))
    expect(await screen.findByText('工程名が長すぎます')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))
    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))

    expect(screen.queryByText('工程名が長すぎます')).not.toBeInTheDocument()
  })

  it('完了に失敗した後で再度完了を試すと前回のエラーメッセージを消す', async () => {
    const responses: MockQueryResponses = {
      CompleteStep: new MockGraphQLError('既に完了している工程です'),
    }
    const { onChanged } = renderActions(createMockClient(responses))

    fireEvent.click(screen.getByRole('button', { name: '工程を完了にする' }))
    expect(await screen.findByText('既に完了している工程です')).toBeInTheDocument()

    // 再試行時は成功レスポンスを返すよう差し替える
    responses.CompleteStep = {
      completeStep: { id: 'step-1', isCompleted: true, completedAt: '2026-01-01T09:30:00+09:00' },
    }
    fireEvent.click(screen.getByRole('button', { name: '工程を完了にする' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
    expect(screen.queryByText('既に完了している工程です')).not.toBeInTheDocument()
  })

  it('開始日時が未設定の工程は入力欄を空欄で開く', () => {
    renderActions(createMockClient({}), buildStep({ startedAt: null }))

    fireEvent.click(screen.getByRole('button', { name: '工程を編集' }))

    expect(screen.getByLabelText('開始日時')).toHaveValue('')
  })
})
