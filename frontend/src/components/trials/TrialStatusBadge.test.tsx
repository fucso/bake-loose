import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { TrialStatusBadge } from './TrialStatusBadge'

describe('TrialStatusBadge', () => {
  it('記録中のステータスを表示する', () => {
    render(<TrialStatusBadge status="IN_PROGRESS" />)

    expect(screen.getByText('記録中')).toBeInTheDocument()
  })

  it('完了のステータスを表示する', () => {
    render(<TrialStatusBadge status="COMPLETED" />)

    expect(screen.getByText('完了')).toBeInTheDocument()
  })
})
