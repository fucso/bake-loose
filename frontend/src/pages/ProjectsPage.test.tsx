import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Provider } from 'urql'
import { MemoryRouter } from 'react-router-dom'
import { createMockClient, MockGraphQLError } from '../../test/mocks/urql'
import ProjectsPage from './ProjectsPage'

const renderProjectsPage = (client: ReturnType<typeof createMockClient>) => {
  return render(
    <Provider value={client}>
      <MemoryRouter>
        <ProjectsPage />
      </MemoryRouter>
    </Provider>,
  )
}

describe('ProjectsPage', () => {
  it('プロジェクト一覧をカード表示し、カードが詳細ページへのリンクになっている', async () => {
    const client = createMockClient({
      Projects: {
        projects: [
          { id: '1', name: 'ピザ生地研究' },
          { id: '2', name: '食パン研究' },
        ],
      },
    })

    renderProjectsPage(client)

    await waitFor(() => {
      expect(screen.getByText('ピザ生地研究')).toBeInTheDocument()
    })
    expect(screen.getByText('食パン研究')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'ピザ生地研究' })).toHaveAttribute(
      'href',
      '/projects/1',
    )
  })

  it('プロジェクトが0件の場合は空状態と作成導線を表示する', async () => {
    const client = createMockClient({
      Projects: { projects: [] },
    })

    renderProjectsPage(client)

    await waitFor(() => {
      expect(screen.getByText('まだプロジェクトがありません')).toBeInTheDocument()
    })
  })

  it('取得に失敗した場合はエラー状態を再試行ボタンとともに表示する', async () => {
    const client = createMockClient({
      Projects: new MockGraphQLError('network error'),
    })

    renderProjectsPage(client)

    await waitFor(() => {
      expect(screen.getByText('プロジェクトの取得に失敗しました')).toBeInTheDocument()
    })
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('新規作成ボタンからモーダルを開いてプロジェクトを作成すると一覧が更新される', async () => {
    const client = createMockClient({
      Projects: { projects: [] },
      CreateProject: {
        createProject: { id: '3', name: '新しいプロジェクト' },
      },
    })

    renderProjectsPage(client)

    await waitFor(() => {
      expect(screen.getByText('まだプロジェクトがありません')).toBeInTheDocument()
    })

    fireEvent.click(screen.getAllByRole('button', { name: '+ 新規作成' })[0])

    const input = await screen.findByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: '新しいプロジェクト' } })
    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    await waitFor(() => {
      expect(screen.queryByText('新規プロジェクト作成')).not.toBeInTheDocument()
    })
  })
})
