import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Kind, type OperationDefinitionNode } from 'graphql'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { Client, Provider, type Exchange } from 'urql'
import { describe, expect, it } from 'vitest'
import { delay, map, pipe } from 'wonka'

import TrialDetailPage from './TrialDetailPage'
import { createMockClient, MockGraphQLError } from '../../test/mocks/urql'

const trialResponse = {
  trial: {
    id: 'trial-1',
    projectId: 'project-1',
    name: '加水率70%',
    memo: '室温24度',
    status: 'IN_PROGRESS',
    completedAt: null,
    steps: [
      {
        id: 'step-2',
        name: '一次発酵',
        position: 1,
        startedAt: null,
        completedAt: null,
        isCompleted: false,
        parameters: [
          {
            id: 'param-2',
            parameterType: 'DURATION',
            content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
          },
        ],
      },
      {
        id: 'step-1',
        name: 'こね',
        position: 0,
        startedAt: '2026-01-01T09:00:00+09:00',
        completedAt: '2026-01-01T09:30:00+09:00',
        isCompleted: true,
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
        ],
      },
    ],
  },
}

/**
 * レスポンスを非同期に返すモッククライアント。
 *
 * 共通の `createMockClient` は同期的に解決するため `fetching` 状態が描画されず、
 * 「再取得中の表示」を検証できない。取得の前後関係が意味を持つテストでのみ使う。
 */
const createDelayedClient = (responses: Record<string, unknown>) => {
  const exchange: Exchange = () => (ops$) =>
    pipe(
      ops$,
      map((operation) => {
        const definition = operation.query.definitions.find(
          (def): def is OperationDefinitionNode => def.kind === Kind.OPERATION_DEFINITION,
        )
        const operationName = definition?.name?.value
        return {
          operation,
          data: operationName ? responses[operationName] : undefined,
          error: undefined,
          extensions: undefined,
          hasNext: false,
          stale: false,
        }
      }),
      delay(10),
    )

  return new Client({ url: 'http://mock.test/graphql', exchanges: [exchange] })
}

const renderPage = (
  client: ReturnType<typeof createMockClient>,
  projectId = 'project-1',
  trialId = 'trial-1',
) =>
  render(
    <Provider value={client}>
      <MemoryRouter initialEntries={[`/projects/${projectId}/trials/${trialId}`]}>
        <Routes>
          <Route path="/projects/:id/trials/:trialId" element={<TrialDetailPage />} />
        </Routes>
      </MemoryRouter>
    </Provider>,
  )

describe('TrialDetailPage', () => {
  it('Trialの情報と工程をposition順のタイムラインで表示する', async () => {
    renderPage(createMockClient({ Trial: trialResponse }))

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '加水率70%' })).toBeInTheDocument()
    })
    expect(screen.getByText('室温24度')).toBeInTheDocument()
    expect(screen.getByText('記録中')).toBeInTheDocument()

    const items = screen.getAllByRole('listitem')
    expect(items[0]).toHaveTextContent('こね')
    expect(items[1]).toHaveTextContent('一次発酵')
  })

  it('記録中の工程のパラメーターを表示し、完了済みの工程は畳む', async () => {
    renderPage(createMockClient({ Trial: trialResponse }))

    await waitFor(() => {
      expect(screen.getByText('90分')).toBeVisible()
    })
    expect(screen.getByText('300g')).not.toBeVisible()
  })

  it('存在しないtrialIdの場合はエラー状態を表示する', async () => {
    renderPage(createMockClient({ Trial: { trial: null } }))

    await waitFor(() => {
      expect(screen.getByText('指定された試行が見つかりません')).toBeInTheDocument()
    })
  })

  it('取得に失敗した場合はエラー状態を再試行ボタンとともに表示する', async () => {
    renderPage(createMockClient({ Trial: new MockGraphQLError('network error') }))

    await waitFor(() => {
      expect(screen.getByText('試行の取得に失敗しました')).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('戻るリンクがプロジェクト詳細のパスを指す', () => {
    renderPage(createMockClient({ Trial: trialResponse }))

    expect(screen.getByRole('link', { name: '← 戻る' })).toHaveAttribute(
      'href',
      '/projects/project-1',
    )
  })

  it('URLのプロジェクトIDが実際の所属と異なる場合はTrialのprojectIdへ戻る', async () => {
    renderPage(createMockClient({ Trial: trialResponse }), 'wrong-project')

    await waitFor(() => {
      expect(screen.getByRole('link', { name: '← 戻る' })).toHaveAttribute(
        'href',
        '/projects/project-1',
      )
    })
  })

  it('Trialの更新後の再取得で内容を表示し続け、工程カードの開閉状態を維持する', async () => {
    const client = createDelayedClient({
      Trial: trialResponse,
      UpdateTrial: { updateTrial: { id: 'trial-1', name: '加水率70%', memo: '室温24度' } },
    })
    renderPage(client)

    // 初回取得中はスピナーを表示する
    expect(screen.getByRole('status')).toBeInTheDocument()

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '加水率70%' })).toBeInTheDocument()
    })

    // 畳まれている完了済みの工程を手動で展開する
    fireEvent.click(screen.getByRole('button', { name: /こね/ }))
    expect(screen.getByText('300g')).toBeVisible()

    // 保存すると再取得が走る。その間も内容は表示され続け、開閉状態も維持される
    fireEvent.click(screen.getByRole('button', { name: '編集' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: '編集' })).toBeInTheDocument()
    })
    expect(screen.queryByRole('status')).not.toBeInTheDocument()
    expect(screen.getByText('300g')).toBeVisible()
  })
})
