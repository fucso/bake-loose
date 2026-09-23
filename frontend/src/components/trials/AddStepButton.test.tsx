import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { Provider } from 'urql'
import { describe, expect, it, vi } from 'vitest'

import { AddStepButton } from './AddStepButton'
import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'

const addStepResponse = {
  addStep: { id: 'step-2', name: '一次発酵', position: 1, startedAt: null },
}

const renderButton = (client: ReturnType<typeof createMockClient>) => {
  const onAdded = vi.fn()
  const utils = render(
    <Provider value={client}>
      <AddStepButton trialId="trial-1" onAdded={onAdded} />
    </Provider>,
  )
  return { ...utils, onAdded }
}

const openForm = () => {
  fireEvent.click(screen.getByRole('button', { name: '+ 工程を追加' }))
}

describe('AddStepButton', () => {
  it('初期状態では入力フォームを表示しない', () => {
    renderButton(createMockClient({}))

    expect(screen.queryByLabelText('工程名')).not.toBeInTheDocument()
  })

  it('追加操作から工程名を入力して追加するとonAddedが呼ばれフォームが閉じる', async () => {
    const { onAdded } = renderButton(createMockClient({ AddStep: addStepResponse }))

    openForm()
    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: '一次発酵' } })
    fireEvent.click(screen.getByRole('button', { name: '追加' }))

    await waitFor(() => {
      expect(onAdded).toHaveBeenCalledTimes(1)
    })
    expect(screen.queryByLabelText('工程名')).not.toBeInTheDocument()
  })

  it('工程名が未入力の場合は追加せずバリデーションエラーを表示する', () => {
    const { onAdded } = renderButton(createMockClient({ AddStep: addStepResponse }))

    openForm()
    fireEvent.click(screen.getByRole('button', { name: '追加' }))

    expect(screen.getByText('工程名を入力してください')).toBeInTheDocument()
    expect(onAdded).not.toHaveBeenCalled()
  })

  it('追加に失敗した場合はエラーメッセージを表示しフォームを開いたままにする', async () => {
    const { onAdded } = renderButton(
      createMockClient({ AddStep: new MockGraphQLError('完了した試行には工程を追加できません') }),
    )

    openForm()
    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: '一次発酵' } })
    fireEvent.click(screen.getByRole('button', { name: '追加' }))

    expect(
      await screen.findByText('完了した試行には工程を追加できません'),
    ).toBeInTheDocument()
    expect(screen.getByLabelText('工程名')).toBeInTheDocument()
    expect(onAdded).not.toHaveBeenCalled()
  })

  it('失敗したフォームを閉じて開き直すと前回のエラーメッセージを持ち越さない', async () => {
    renderButton(
      createMockClient({ AddStep: new MockGraphQLError('完了した試行には工程を追加できません') }),
    )

    openForm()
    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: '一次発酵' } })
    fireEvent.click(screen.getByRole('button', { name: '追加' }))
    expect(
      await screen.findByText('完了した試行には工程を追加できません'),
    ).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))
    openForm()

    expect(
      screen.queryByText('完了した試行には工程を追加できません'),
    ).not.toBeInTheDocument()
  })
})
