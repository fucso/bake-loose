import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { StepCard } from './StepCard'
import type { Step } from '@/lib/trial'

const buildStep = (overrides: Partial<Step> = {}): Step => ({
  id: 'step-1',
  name: 'こね',
  position: 0,
  startedAt: '2026-01-01T09:00:00+09:00',
  completedAt: null,
  isCompleted: false,
  parameters: [
    {
      id: 'param-1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    },
  ],
  ...overrides,
})

describe('StepCard', () => {
  it('折りたたみ時もヘッダのサマリーは表示し、パラメーターは隠す', () => {
    render(<StepCard step={buildStep()} isCurrent={false} defaultExpanded={false} />)

    expect(screen.getByText('こね')).toBeVisible()
    expect(screen.getByText(/パラメーター 1件/)).toBeVisible()
    expect(screen.getByText('強力粉')).not.toBeVisible()
  })

  it('展開状態ではパラメーターを表示する', () => {
    render(<StepCard step={buildStep()} isCurrent defaultExpanded />)

    expect(screen.getByText('強力粉')).toBeVisible()
    expect(screen.getByText('300g')).toBeVisible()
  })

  it('ヘッダのクリックで開閉を切り替えられる', () => {
    render(<StepCard step={buildStep()} isCurrent={false} defaultExpanded={false} />)

    const toggle = screen.getByRole('button')
    expect(toggle).toHaveAttribute('aria-expanded', 'false')

    fireEvent.click(toggle)

    expect(toggle).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByText('強力粉')).toBeVisible()
  })

  it('記録中の工程であることを示すラベルを表示する', () => {
    render(<StepCard step={buildStep()} isCurrent defaultExpanded />)

    expect(screen.getByText('進行中')).toBeInTheDocument()
  })

  it('完了済みの工程は完了ラベルと完了日時を表示する', () => {
    const step = buildStep({ isCompleted: true, completedAt: '2026-01-01T09:30:00+09:00' })

    render(<StepCard step={step} isCurrent={false} defaultExpanded={false} />)

    expect(screen.getByText('完了')).toBeInTheDocument()
    expect(screen.getByText(/完了 2026\/01\/01 09:30/)).toBeInTheDocument()
  })

  it('パラメーターが無い工程には未記録であることを表示する', () => {
    render(
      <StepCard step={buildStep({ parameters: [] })} isCurrent={false} defaultExpanded />,
    )

    expect(screen.getByText('パラメーターは記録されていません')).toBeVisible()
  })
})
