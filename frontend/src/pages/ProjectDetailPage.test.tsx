import { render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Provider } from 'urql'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { createMockClient, MockGraphQLError } from '../../test/mocks/urql'
import ProjectDetailPage from './ProjectDetailPage'

function renderDetailPage(client: ReturnType<typeof createMockClient>, id = '1') {
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

describe('ProjectDetailPage', () => {
  it('プロジェクト名を表示する', async () => {
    const client = createMockClient({
      Project: { project: { id: '1', name: 'ピザ生地研究' } },
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('ピザ生地研究')).toBeInTheDocument()
    })
  })

  it('存在しないIDの場合はエラー状態を表示する', async () => {
    const client = createMockClient({
      Project: { project: null },
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('指定されたプロジェクトが見つかりません')).toBeInTheDocument()
    })
  })

  it('取得に失敗した場合はエラー状態を再試行ボタンとともに表示する', async () => {
    const client = createMockClient({
      Project: new MockGraphQLError('network error'),
    })

    renderDetailPage(client)

    await waitFor(() => {
      expect(screen.getByText('プロジェクトの取得に失敗しました')).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('戻るリンクが一覧ページのパスを指す', () => {
    const client = createMockClient({
      Project: { project: { id: '1', name: 'ピザ生地研究' } },
    })

    renderDetailPage(client)

    expect(screen.getByRole('link', { name: '← 戻る' })).toHaveAttribute('href', '/projects')
  })
})
