import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it } from 'vitest'

import { TrialCard, type Trial } from './TrialCard'

const baseTrial: Trial = {
  id: 'trial-1',
  name: '加水率70%',
  status: 'IN_PROGRESS',
  completedAt: null,
}

const renderCard = (trial: Trial) => {
  return render(
    <MemoryRouter>
      <TrialCard trial={trial} projectId="project-1" />
    </MemoryRouter>,
  )
}

describe('TrialCard', () => {
  it('Trial名とステータスバッジを表示する', () => {
    renderCard(baseTrial)

    expect(screen.getByText('加水率70%')).toBeInTheDocument()
    expect(screen.getByText('進行中')).toBeInTheDocument()
  })

  it('name未設定の場合はフォールバックラベルを表示する', () => {
    renderCard({ ...baseTrial, name: null })

    expect(screen.getByText('名称未設定の試行')).toBeInTheDocument()
  })

  it('完了済みの場合は完了バッジと完了日時を表示する', () => {
    renderCard({
      ...baseTrial,
      status: 'COMPLETED',
      completedAt: '2026-03-01T09:30:00+09:00',
    })

    expect(screen.getByText('完了')).toBeInTheDocument()
    expect(screen.getByText('完了 2026/03/01 09:30')).toBeInTheDocument()
  })

  it('カードがTrial詳細画面へのリンクになっている', () => {
    renderCard(baseTrial)

    expect(screen.getByRole('link')).toHaveAttribute(
      'href',
      '/projects/project-1/trials/trial-1',
    )
  })
})
