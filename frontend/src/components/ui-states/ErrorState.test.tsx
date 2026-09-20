import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { ErrorState } from './ErrorState'

describe('ErrorState', () => {
  it('onRetry未指定時は再試行ボタンが表示されない', () => {
    render(<ErrorState message="エラーが発生しました" />)

    expect(screen.getByText('エラーが発生しました')).toBeInTheDocument()
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('onRetry指定時にボタンクリックでハンドラが呼ばれる', () => {
    const handleRetry = vi.fn()
    render(<ErrorState message="エラーが発生しました" onRetry={handleRetry} />)

    fireEvent.click(screen.getByRole('button', { name: '再試行' }))

    expect(handleRetry).toHaveBeenCalledTimes(1)
  })
})
