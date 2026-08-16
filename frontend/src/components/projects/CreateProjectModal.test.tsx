import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { Provider } from 'urql'
import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import { CreateProjectModal } from './CreateProjectModal'

function renderModal(client: ReturnType<typeof createMockClient>) {
  const onOpenChange = vi.fn()
  const onCreated = vi.fn()
  const utils = render(
    <Provider value={client}>
      <CreateProjectModal open onOpenChange={onOpenChange} onCreated={onCreated} />
    </Provider>,
  )
  return { ...utils, onOpenChange, onCreated }
}

describe('CreateProjectModal', () => {
  it('name未入力で送信するとバリデーションエラーを表示し作成しない', async () => {
    const client = createMockClient({})
    const { onCreated } = renderModal(client)

    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    expect(await screen.findByText('プロジェクト名を入力してください')).toBeInTheDocument()
    expect(onCreated).not.toHaveBeenCalled()
  })

  it('作成に成功するとonCreatedが呼ばれる', async () => {
    const client = createMockClient({
      CreateProject: { createProject: { id: '1', name: '新しいプロジェクト' } },
    })
    const { onCreated } = renderModal(client)

    fireEvent.change(screen.getByLabelText('プロジェクト名'), {
      target: { value: '新しいプロジェクト' },
    })
    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    await waitFor(() => {
      expect(onCreated).toHaveBeenCalledTimes(1)
    })
  })

  it('作成に失敗するとエラーメッセージを表示しonCreatedは呼ばれない', async () => {
    const client = createMockClient({
      CreateProject: new MockGraphQLError('failed'),
    })
    const { onCreated } = renderModal(client)

    fireEvent.change(screen.getByLabelText('プロジェクト名'), {
      target: { value: '新しいプロジェクト' },
    })
    fireEvent.click(screen.getByRole('button', { name: '作成' }))

    expect(await screen.findByText('プロジェクトの作成に失敗しました')).toBeInTheDocument()
    expect(onCreated).not.toHaveBeenCalled()
  })

  it('キャンセルボタンでonOpenChangeがfalseで呼ばれる', () => {
    const client = createMockClient({})
    const { onOpenChange } = renderModal(client)

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
