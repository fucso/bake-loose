import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Provider, type Client } from 'urql'
import { describe, expect, it, vi } from 'vitest'
import { fromPromise } from 'wonka'

import { TrialHeader } from './TrialHeader'
import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import type { Trial } from '@/lib/trial'

const buildTrial = (overrides: Partial<Trial> = {}): Trial => ({
  id: 'trial-1',
  projectId: 'project-1',
  name: '加水率70%',
  memo: '室温24度',
  status: 'IN_PROGRESS',
  completedAt: null,
  steps: [],
  ...overrides,
})

const renderHeader = (
  client: ReturnType<typeof createMockClient>,
  trial: Trial = buildTrial(),
) => {
  const onChanged = vi.fn()
  const utils = render(
    <Provider value={client}>
      <TrialHeader trial={trial} onChanged={onChanged} />
    </Provider>,
  )
  return { ...utils, onChanged }
}

describe('TrialHeader', () => {
  it('name/memo/statusを表示する', () => {
    renderHeader(createMockClient({}))

    expect(screen.getByRole('heading', { name: '加水率70%' })).toBeInTheDocument()
    expect(screen.getByText('室温24度')).toBeInTheDocument()
    expect(screen.getByText('記録中')).toBeInTheDocument()
  })

  it('nameが未設定の場合はフォールバック表示をする', () => {
    renderHeader(createMockClient({}), buildTrial({ name: null }))

    expect(screen.getByRole('heading', { name: '名称未設定の試行' })).toBeInTheDocument()
  })

  it('編集して保存するとupdateTrialが成功しonChangedが呼ばれる', async () => {
    const client = createMockClient({
      UpdateTrial: { updateTrial: { id: 'trial-1', name: '加水率75%', memo: '室温24度' } },
    })
    const { onChanged } = renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '編集' }))
    fireEvent.change(screen.getByLabelText('試行名'), { target: { value: '加水率75%' } })
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
    // 保存に成功すると表示モードへ戻る
    expect(screen.getByRole('button', { name: '編集' })).toBeInTheDocument()
  })

  it('更新に失敗した場合はエラーメッセージを表示しonChangedは呼ばれない', async () => {
    const client = createMockClient({
      UpdateTrial: new MockGraphQLError('試行名が長すぎます'),
    })
    const { onChanged } = renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '編集' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    expect(await screen.findByText('試行名が長すぎます')).toBeInTheDocument()
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('編集をキャンセルすると表示モードへ戻る', () => {
    renderHeader(createMockClient({}))

    fireEvent.click(screen.getByRole('button', { name: '編集' }))
    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(screen.getByRole('heading', { name: '加水率70%' })).toBeInTheDocument()
  })

  it('完了にするとcompleteTrialが成功しonChangedが呼ばれる', async () => {
    const client = createMockClient({
      CompleteTrial: {
        completeTrial: {
          id: 'trial-1',
          status: 'COMPLETED',
          completedAt: '2026-01-01T12:00:00+09:00',
        },
      },
    })
    const { onChanged } = renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '完了にする' }))

    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })
  })

  it('完了に失敗した場合はエラーメッセージを表示する', async () => {
    const client = createMockClient({
      CompleteTrial: new MockGraphQLError('既に完了している試行です'),
    })
    const { onChanged } = renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '完了にする' }))

    expect(await screen.findByText('既に完了している試行です')).toBeInTheDocument()
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('完了済みTrialは完了日時を表示し完了操作を出さない', () => {
    renderHeader(
      createMockClient({}),
      buildTrial({ status: 'COMPLETED', completedAt: '2026-01-01T12:00:00+09:00' }),
    )

    expect(screen.getByText('完了日時: 2026/01/01 12:00')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '完了にする' })).not.toBeInTheDocument()
  })

  it('完了済みのTrialには編集と完了の操作を出さない', () => {
    renderHeader(createMockClient({}), buildTrial({ status: 'COMPLETED', completedAt: '2026-01-01T10:00:00+09:00' }))

    expect(screen.queryByRole('button', { name: '編集' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '完了にする' })).not.toBeInTheDocument()
  })

  it('更新失敗後にキャンセルして開き直すと前回のエラーを表示しない', async () => {
    const client = createMockClient({
      UpdateTrial: new MockGraphQLError('名称が重複しています'),
    })
    renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '編集' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('名称が重複しています')

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))
    fireEvent.click(screen.getByRole('button', { name: '編集' }))

    expect(screen.queryByText('名称が重複しています')).not.toBeInTheDocument()
  })

  it('完了に成功した後は再取得を待つ間も完了操作を押せない', async () => {
    const client = createMockClient({
      CompleteTrial: { completeTrial: { id: 'trial-1', status: 'COMPLETED', completedAt: '2026-01-01T10:00:00+09:00' } },
    })
    const { onChanged } = renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '完了にする' }))
    await waitFor(() => {
      expect(onChanged).toHaveBeenCalledTimes(1)
    })

    expect(screen.getByRole('button', { name: '完了にする' })).toBeDisabled()
    // 再取得が終わるまでの間も、完了済みになる Trial の編集は出さない
    expect(screen.queryByRole('button', { name: '編集' })).not.toBeInTheDocument()
  })

  it('完了処理中は編集を開けず、完了後に編集フォームが残らない', async () => {
    let resolveResponse!: (data: unknown) => void
    const response = new Promise((resolve) => {
      resolveResponse = resolve
    })
    const client = {
      executeMutation: vi.fn((operation) =>
        fromPromise(
          response.then((data) => ({ operation, data, stale: false, hasNext: false })),
        ),
      ),
      executeQuery: vi.fn(),
      executeSubscription: vi.fn(),
    } as unknown as Client
    renderHeader(client)

    fireEvent.click(screen.getByRole('button', { name: '完了にする' }))

    expect(screen.getByRole('button', { name: '編集' })).toBeDisabled()

    await act(async () => {
      resolveResponse({
        completeTrial: { id: 'trial-1', status: 'COMPLETED', completedAt: '2026-01-01T10:00:00+09:00' },
      })
      await response
    })
    expect(screen.queryByRole('button', { name: '保存' })).not.toBeInTheDocument()
  })
})
