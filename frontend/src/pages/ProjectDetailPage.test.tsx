import { render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Provider } from 'urql'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { createMockClient, MockGraphQLError } from '../../test/mocks/urql'
import ProjectDetailPage from './ProjectDetailPage'

const renderDetailPage = (client: ReturnType<typeof createMockClient>, id = '1') => {
  return render(
    <Provider value={client}>
      <MemoryRouter initialEntries={[`/projects/${id}`]}>
        <Routes>
          <Route path="/projects/:id" element={<ProjectDetailPage />} />
        </Routes>
      </MemoryRouter>
    </Provider>,
  )
}

/** Trial 一覧は別コンポーネントで検証するため、既定では空の一覧を返す */
const projectResponses = {
  Project: { project: { id: '1', name: 'ピザ生地研究' } },
  TrialsByProject: { trialsByProject: [] },
}

describe('ProjectDetailPage', () => {
  it('プロジェクト名を表示する', async () => {
    const client = createMockClient(projectResponses)

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('ピザ生地研究')).toBeInTheDocument()
    })
  })

  it('プロジェクトに紐づくTrial一覧セクションを表示する', async () => {
    const client = createMockClient({
      ...projectResponses,
      TrialsByProject: {
        trialsByProject: [
          { id: 'trial-1', name: '加水率70%', status: 'IN_PROGRESS', completedAt: null },
        ],
      },
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '試行' })).toBeInTheDocument()
    })
    expect(screen.getByRole('link', { name: /加水率70%/ })).toHaveAttribute(
      'href',
      '/projects/1/trials/trial-1',
    )
  })

  it('存在しないIDの場合はエラー状態を表示する', async () => {
    const client = createMockClient({
      ...projectResponses,
      Project: { project: null },
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('指定されたプロジェクトが見つかりません')).toBeInTheDocument()
    })
    expect(screen.queryByRole('heading', { name: '試行' })).not.toBeInTheDocument()
  })

  it('取得に失敗した場合はエラー状態を再試行ボタンとともに表示する', async () => {
    const client = createMockClient({
      ...projectResponses,
      Project: new MockGraphQLError('network error'),
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('プロジェクトの取得に失敗しました')).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('戻るリンクが一覧ページのパスを指す', () => {
    const client = createMockClient(projectResponses)

    renderDetailPage(client)

    expect(screen.getByRole('link', { name: '← 戻る' })).toHaveAttribute('href', '/projects')
  })
})
