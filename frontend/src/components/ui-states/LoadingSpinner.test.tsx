import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { LoadingSpinner } from './LoadingSpinner'

describe('LoadingSpinner', () => {
  it('messageなしでレンダリングされる', () => {
    const { container } = render(<LoadingSpinner />)

    expect(screen.getByRole('status')).toBeInTheDocument()
    expect(container.querySelector('p')).not.toBeInTheDocument()
  })

  it('message指定時にテキストが表示される', () => {
    render(<LoadingSpinner message="読み込み中..." />)

    expect(screen.getByText('読み込み中...')).toBeInTheDocument()
  })
})
