import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { StepFormModal, type StepFormModalProps } from './StepFormModal'

const renderModal = (props: Partial<StepFormModalProps> = {}) => {
  const onSubmit = vi.fn()
  const onOpenChange = vi.fn()
  const utils = render(
    <StepFormModal
      open
      onOpenChange={onOpenChange}
      title="工程を追加"
      submitLabel="追加"
      initialName=""
      initialStartedAt={null}
      submitting={false}
      errorMessage={null}
      onSubmit={onSubmit}
      {...props}
    />,
  )
  return { ...utils, onSubmit, onOpenChange }
}

describe('StepFormModal', () => {
  it('初期値を入力欄に表示する', () => {
    renderModal({ initialName: 'こね', initialStartedAt: '2026-01-01T09:00:00+09:00' })

    expect(screen.getByLabelText('工程名')).toHaveValue('こね')
    expect(screen.getByLabelText('開始日時')).toHaveValue('2026-01-01T09:00')
  })

  it('工程名が未入力の場合はバリデーションエラーを表示し送信しない', () => {
    const { onSubmit } = renderModal()

    fireEvent.click(screen.getByRole('button', { name: '追加' }))

    expect(screen.getByText('工程名を入力してください')).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('工程名と開始日時をJSTのISO文字列として送信する', () => {
    const { onSubmit } = renderModal()

    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: ' こね ' } })
    fireEvent.change(screen.getByLabelText('開始日時'), {
      target: { value: '2026-01-01T09:00' },
    })
    fireEvent.click(screen.getByRole('button', { name: '追加' }))

    expect(onSubmit).toHaveBeenCalledWith({
      name: 'こね',
      startedAt: '2026-01-01T09:00:00+09:00',
    })
  })

  it('開始日時を空欄にするとクリアの意図としてnullを送信する', () => {
    const { onSubmit } = renderModal({
      initialName: 'こね',
      initialStartedAt: '2026-01-01T09:00:00+09:00',
      submitLabel: '保存',
    })

    fireEvent.change(screen.getByLabelText('開始日時'), { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    expect(onSubmit).toHaveBeenCalledWith({ name: 'こね', startedAt: null })
  })

  it('送信中は入力と操作を無効化する', () => {
    renderModal({ submitting: true })

    expect(screen.getByLabelText('工程名')).toBeDisabled()
    expect(screen.getByLabelText('開始日時')).toBeDisabled()
    expect(screen.getByRole('button', { name: '追加' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'キャンセル' })).toBeDisabled()
  })

  it('送信に失敗した場合は呼び出し元から渡されたエラーメッセージを表示する', () => {
    renderModal({ errorMessage: '工程名が長すぎます' })

    expect(screen.getByText('工程名が長すぎます')).toBeInTheDocument()
  })

  it('キャンセルするとonOpenChangeがfalseで呼ばれる', () => {
    const { onOpenChange } = renderModal()

    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('Escキーで閉じられる', () => {
    const { onOpenChange } = renderModal()

    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })

    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('送信中はEscキーで閉じない（送信結果の表示先を失わないため）', () => {
    const { onOpenChange } = renderModal({ submitting: true })

    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })

    expect(onOpenChange).not.toHaveBeenCalled()
  })

  it('開き直すとその時点の初期値でフォームを作り直す', () => {
    const { rerender, onSubmit } = renderModal({ initialName: 'こね', initialStartedAt: null })

    fireEvent.change(screen.getByLabelText('工程名'), { target: { value: '入力途中' } })

    const props = {
      onOpenChange: vi.fn(),
      title: '工程を編集',
      submitLabel: '保存',
      initialStartedAt: null,
      submitting: false,
      errorMessage: null,
      onSubmit,
    }
    // 閉じている間に再取得で名前が変わっても、開き直せば最新の値が表示される
    rerender(<StepFormModal {...props} open={false} initialName="こね（改）" />)
    rerender(<StepFormModal {...props} open initialName="こね（改）" />)

    expect(screen.getByLabelText('工程名')).toHaveValue('こね（改）')
  })
})
