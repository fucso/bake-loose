import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { StepTimeline } from './StepTimeline'
import type { Step } from '@/lib/trial'

const buildStep = (overrides: Partial<Step> & Pick<Step, 'id' | 'position'>): Step => ({
  name: `工程${overrides.position}`,
  startedAt: null,
  completedAt: null,
  isCompleted: false,
  parameters: [],
  ...overrides,
})

describe('StepTimeline', () => {
  it('工程が無い場合は空状態を表示する', () => {
    render(<StepTimeline steps={[]} trialStatus="IN_PROGRESS" />)

    expect(screen.getByText('まだ工程が記録されていません')).toBeInTheDocument()
  })

  it('position昇順で表示する', () => {
    const steps = [
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
      buildStep({ id: 'a', position: 0, name: 'こね' }),
    ]

    render(<StepTimeline steps={steps} trialStatus="IN_PROGRESS" />)

    const items = screen.getAllByRole('listitem')
    expect(items[0]).toHaveTextContent('こね')
    expect(items[1]).toHaveTextContent('一次発酵')
  })

  it('最初の未完了工程を記録中の工程として示す', () => {
    const steps = [
      buildStep({ id: 'a', position: 0, name: 'こね', isCompleted: true }),
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
      buildStep({ id: 'c', position: 2, name: '焼成' }),
    ]

    render(<StepTimeline steps={steps} trialStatus="IN_PROGRESS" />)

    const items = screen.getAllByRole('listitem')
    expect(items[1]).toHaveAttribute('aria-current', 'step')
    expect(items[0]).not.toHaveAttribute('aria-current')
    expect(items[2]).not.toHaveAttribute('aria-current')
  })

  it('記録中の工程は展開し、完了済みの工程は畳む', () => {
    const steps = [
      buildStep({
        id: 'a',
        position: 0,
        name: 'こね',
        isCompleted: true,
        parameters: [
          { id: 'p1', parameterType: 'TEXT', content: { type: 'text', value: '完了済みメモ' } },
        ],
      }),
      buildStep({
        id: 'b',
        position: 1,
        name: '一次発酵',
        parameters: [
          { id: 'p2', parameterType: 'TEXT', content: { type: 'text', value: '記録中メモ' } },
        ],
      }),
    ]

    render(<StepTimeline steps={steps} trialStatus="IN_PROGRESS" />)

    expect(screen.getByText('完了済みメモ')).not.toBeVisible()
    expect(screen.getByText('記録中メモ')).toBeVisible()
  })

  it('完了済みTrialでは記録中の工程を持たず、全工程を畳んで表示する', () => {
    const steps = [
      buildStep({
        id: 'a',
        position: 0,
        name: 'こね',
        parameters: [
          { id: 'p1', parameterType: 'TEXT', content: { type: 'text', value: 'メモ' } },
        ],
      }),
    ]

    render(<StepTimeline steps={steps} trialStatus="COMPLETED" />)

    expect(screen.getByRole('listitem')).not.toHaveAttribute('aria-current')
    expect(screen.getByText('メモ')).not.toBeVisible()
  })
})
