import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Client, Provider, type Exchange } from 'urql'
import { map, pipe } from 'wonka'
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

  it('キャンセルボタンでonOpenChangeがfalseで呼ばれる', () => {
    const { onOpenChange } = renderSheet(createMockClient(CREATE_TRIAL_SUCCESS))

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
