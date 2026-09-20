import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Pizza } from 'lucide-react'
import { EmptyState } from './EmptyState'

describe('EmptyState', () => {
  it('デフォルトアイコン（Inbox）でレンダリングされる', () => {
    const { container } = render(<EmptyState message="まだ記録がありません" />)

    expect(screen.getByText('まだ記録がありません')).toBeInTheDocument()
    expect(container.querySelector('svg.lucide-inbox')).toBeInTheDocument()
  })

  it('icon/actionを指定した場合にカスタマイズが反映される', () => {
    const { container } = render(
      <EmptyState
        message="該当する結果がありません"
        icon={Pizza}
        action={<button type="button">最初のTrialを記録する</button>}
      />,
    )

    expect(container.querySelector('svg.lucide-inbox')).not.toBeInTheDocument()
    expect(container.querySelector('svg.lucide-pizza')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '最初のTrialを記録する' })).toBeInTheDocument()
  })
})
