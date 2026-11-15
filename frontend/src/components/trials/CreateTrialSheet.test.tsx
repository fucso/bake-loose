import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { Client, CombinedError, Provider, type Exchange } from 'urql'
import { fromPromise, map, mergeMap, pipe } from 'wonka'
import { describe, expect, it, vi } from 'vitest'

import {
  createMockClient,
  createMockExchange,
  MockGraphQLError,
  type MockQueryResponses,
} from '../../../test/mocks/urql'
import { CreateTrialSheet } from './CreateTrialSheet'

const CREATE_TRIAL_SUCCESS: MockQueryResponses = {
  CreateTrial: { createTrial: { id: 'trial-1' } },
}

/**
 * 実際に送信された variables を記録しつつ、モックレスポンスを返す Client を生成する。
 * mutation に渡す input の組み立て（空入力を null に変換する等）を検証するために使用する。
 */
const createRecordingClient = (responses: MockQueryResponses, recorded: unknown[]) => {
  const recordExchange: Exchange =
    ({ forward }) =>
    (ops$) =>
      forward(
        pipe(
          ops$,
          map((operation) => {
            recorded.push(operation.variables)
            return operation
          }),
        ),
      )

  return new Client({
    url: 'http://mock.test/graphql',
    exchanges: [recordExchange, createMockExchange(responses)],
  })
}

/**
 * fail() を呼ぶまでレスポンスを保留し、呼ばれた時点で GraphQL エラーを返す Client を生成する。
 * mutation の送信中に行われたユーザー操作を検証するために使用する。
 */
const createPendingFailureClient = (message: string) => {
  let fail!: () => void
  const released = new Promise<void>((resolve) => {
    fail = resolve
  })
  const pendingExchange: Exchange = () => (ops$) =>
    pipe(
      ops$,
      mergeMap((operation) =>
        fromPromise(
          released.then(() => ({
            operation,
            data: undefined,
            error: new CombinedError({ graphQLErrors: [message] }),
            extensions: undefined,
            hasNext: false,
            stale: false,
          })),
        ),
      ),
    )

  return {
    client: new Client({ url: 'http://mock.test/graphql', exchanges: [pendingExchange] }),
    fail,
  }
}

/** 呼び出し元と同様に onOpenChange を open に反映し、開閉を実際に切り替える */
const StatefulSheet = () => {
  const [open, setOpen] = useState(true)
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        開く
      </button>
      <CreateTrialSheet
        projectId="project-1"
        open={open}
        onOpenChange={setOpen}
        onCreated={() => setOpen(false)}
      />
    </>
  )
}

const renderSheet = (client: Client, open = true) => {
  const onOpenChange = vi.fn()
  const onCreated = vi.fn()
  const renderSheetWithOpen = (isOpen: boolean) => (
    <Provider value={client}>
      <CreateTrialSheet
        projectId="project-1"
        open={isOpen}
        onOpenChange={onOpenChange}
        onCreated={onCreated}
      />
    </Provider>
  )
  const utils = render(renderSheetWithOpen(open))
  return {
    ...utils,
    onOpenChange,
    onCreated,
    setOpen: (isOpen: boolean) => utils.rerender(renderSheetWithOpen(isOpen)),
  }
}

describe('CreateTrialSheet', () => {
  it('name / memo を未入力のまま作成するとinputにnullを送る', async () => {
    const recorded: unknown[] = []
    const { onCreated } = renderSheet(createRecordingClient(CREATE_TRIAL_SUCCESS, recorded))

    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    await waitFor(() => {
      expect(onCreated).toHaveBeenCalledTimes(1)
    })
    expect(recorded).toEqual([
      { input: { projectId: 'project-1', name: null, memo: null } },
    ])
  })

  it('name / memo を入力して作成できる', async () => {
    const recorded: unknown[] = []
    const { onCreated } = renderSheet(createRecordingClient(CREATE_TRIAL_SUCCESS, recorded))

    fireEvent.change(screen.getByLabelText('試行名（任意）'), {
      target: { value: '  加水率70%  ' },
    })
    fireEvent.change(screen.getByLabelText('メモ（任意）'), {
      target: { value: '室温24度' },
    })
    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    await waitFor(() => {
      expect(onCreated).toHaveBeenCalledTimes(1)
    })
    expect(recorded).toEqual([
      { input: { projectId: 'project-1', name: '加水率70%', memo: '室温24度' } },
    ])
  })

  it('作成に失敗した場合バックエンドのエラーメッセージを表示しonCreatedは呼ばれない', async () => {
    const client = createMockClient({
      CreateTrial: new MockGraphQLError('指定されたプロジェクトが見つかりません'),
    })
    const { onCreated } = renderSheet(client)

    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    expect(
      await screen.findByText('指定されたプロジェクトが見つかりません'),
    ).toBeInTheDocument()
    expect(onCreated).not.toHaveBeenCalled()
  })

  it('失敗後にシートを閉じて開き直すとエラーメッセージが残らない', async () => {
    const client = createMockClient({
      CreateTrial: new MockGraphQLError('指定されたプロジェクトが見つかりません'),
    })
    const { setOpen } = renderSheet(client)

    fireEvent.click(screen.getByRole('button', { name: '作成' }))
    expect(
      await screen.findByText('指定されたプロジェクトが見つかりません'),
    ).toBeInTheDocument()

    // キャンセル相当（onOpenChange(false) → 親が open を false にする）を再現する
    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))
    setOpen(false)
    setOpen(true)

    expect(await screen.findByRole('button', { name: '作成' })).toBeInTheDocument()
    expect(
      screen.queryByText('指定されたプロジェクトが見つかりません'),
    ).not.toBeInTheDocument()
  })

  it('作成中にEscキーを押してもシートは閉じず、失敗時のエラーをシート内に表示する', async () => {
    const { client, fail } = createPendingFailureClient('指定されたプロジェクトが見つかりません')
    render(
      <Provider value={client}>
        <StatefulSheet />
      </Provider>,
    )

    fireEvent.click(screen.getByRole('button', { name: '作成' }))
    await waitFor(() => {
      expect(screen.getByRole('button', { name: '作成' })).toBeDisabled()
    })
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })

    await act(async () => {
      fail()
    })

    expect(
      await screen.findByText('指定されたプロジェクトが見つかりません'),
    ).toBeInTheDocument()
    expect(screen.getByRole('dialog')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })
    fireEvent.click(screen.getByRole('button', { name: '開く' }))

    expect(await screen.findByRole('dialog')).toBeInTheDocument()
    expect(
      screen.queryByText('指定されたプロジェクトが見つかりません'),
    ).not.toBeInTheDocument()
  })

  it('キャンセルボタンでonOpenChangeがfalseで呼ばれる', () => {
    const { onOpenChange } = renderSheet(createMockClient(CREATE_TRIAL_SUCCESS))

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
