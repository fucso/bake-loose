import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { Provider } from 'urql'
import { describe, expect, it } from 'vitest'

import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import { TrialListSection } from './TrialListSection'

const renderSection = (client: ReturnType<typeof createMockClient>) => {
  return render(
    <Provider value={client}>
      <MemoryRouter>
        <TrialListSection projectId="project-1" />
      </MemoryRouter>
    </Provider>,
  )
}

describe('TrialListSection', () => {
  it('取得したTrialをカード一覧として表示する', async () => {
    const client = createMockClient({
      TrialsByProject: {
        trialsByProject: [
          { id: 'trial-1', name: '加水率70%', status: 'IN_PROGRESS', completedAt: null },
          {
            id: 'trial-2',
            name: null,
            status: 'COMPLETED',
            completedAt: '2026-03-01T09:30:00+09:00',
          },
        ],
      },
    })

    renderSection(client)

    await waitFor(() => {
      expect(screen.getByText('加水率70%')).toBeInTheDocument()
    })
    expect(screen.getByText('名称未設定の試行')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /加水率70%/ })).toHaveAttribute(
      'href',
      '/projects/project-1/trials/trial-1',
    )
  })

  it('Trialが0件の場合は空状態を表示する', async () => {
    const client = createMockClient({
      TrialsByProject: { trialsByProject: [] },
    })

    renderSection(client)

    await waitFor(() => {
      expect(screen.getByText('まだ試行がありません')).toBeInTheDocument()
    })
  })

  it('取得に失敗した場合はエラー状態を再試行ボタンとともに表示する', async () => {
    const client = createMockClient({
      TrialsByProject: new MockGraphQLError('network error'),
    })

    renderSection(client)

    await waitFor(() => {
      expect(screen.getByText('試行の取得に失敗しました')).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('新しい試行ボタンで作成シートが開く', async () => {
    const client = createMockClient({
      TrialsByProject: { trialsByProject: [] },
    })

    renderSection(client)

    await waitFor(() => {
      expect(screen.getByText('まだ試行がありません')).toBeInTheDocument()
    })
    expect(screen.queryByText('新しい試行')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '最初の試行を記録する' }))

    expect(await screen.findByText('新しい試行')).toBeInTheDocument()
    expect(screen.getByLabelText('試行名（任意）')).toBeInTheDocument()
    expect(screen.getByLabelText('メモ（任意）')).toBeInTheDocument()
  })

  it('作成に成功するとシートが閉じて一覧に反映される', async () => {
    const client = createMockClient({
      TrialsByProject: {
        trialsByProject: [
          { id: 'trial-1', name: '加水率70%', status: 'IN_PROGRESS', completedAt: null },
        ],
      },
      CreateTrial: { createTrial: { id: 'trial-2' } },
    })

    renderSection(client)

    await waitFor(() => {
      expect(screen.getByText('加水率70%')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByRole('button', { name: '+ 新しい試行' }))
    fireEvent.click(await screen.findByRole('button', { name: '作成' }))

    await waitFor(() => {
      expect(screen.queryByLabelText('試行名（任意）')).not.toBeInTheDocument()
    })
    expect(screen.getByText('加水率70%')).toBeInTheDocument()
  })
})
