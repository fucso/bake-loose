import { render, screen } from '@testing-library/react'
import { Provider } from 'urql'
import { describe, expect, it } from 'vitest'

import { StepTimeline } from './StepTimeline'
import { createMockClient } from '../../../test/mocks/urql'
import type { Step, TrialStatus } from '@/lib/trial'

const buildStep = (overrides: Partial<Step> & Pick<Step, 'id' | 'position'>): Step => ({
  name: `工程${overrides.position}`,
  startedAt: null,
  completedAt: null,
  isCompleted: false,
  parameters: [],
  ...overrides,
})

const renderTimeline = (steps: Step[], trialStatus: TrialStatus) =>
  render(
    <Provider value={createMockClient({})}>
      <StepTimeline
        trialId="trial-1"
        steps={steps}
        trialStatus={trialStatus}
        onChanged={() => {}}
      />
    </Provider>,
  )

describe('StepTimeline', () => {
  it('工程が無い場合は空状態を表示する', () => {
    renderTimeline([], 'IN_PROGRESS')

    expect(screen.getByText('まだ工程が記録されていません')).toBeInTheDocument()
  })

  it('position昇順で表示する', () => {
    const steps = [
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
      buildStep({ id: 'a', position: 0, name: 'こね' }),
    ]

    renderTimeline(steps, 'IN_PROGRESS')

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

    renderTimeline(steps, 'IN_PROGRESS')

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

    renderTimeline(steps, 'IN_PROGRESS')

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

    renderTimeline(steps, 'COMPLETED')

    expect(screen.getAllByRole('listitem')[0]).not.toHaveAttribute('aria-current')
    expect(screen.getByText('メモ')).not.toBeVisible()
  })

  it('未完了の工程にだけパラメーターの記録操作を出す', () => {
    const steps = [
      buildStep({ id: 'a', position: 0, name: 'こね', isCompleted: true }),
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
    ]

    renderTimeline(steps, 'IN_PROGRESS')

    // 完了済みの工程は畳まれているため、操作 UI の有無は件数で確かめる
    expect(screen.getAllByRole('button', { name: 'パラメーター追加' })).toHaveLength(1)
  })

  it('完了済みTrialではパラメーターの記録操作を出さない', () => {
    const steps = [buildStep({ id: 'a', position: 0, name: 'こね' })]

    renderTimeline(steps, 'COMPLETED')

    expect(screen.queryByRole('button', { name: 'パラメーター追加' })).not.toBeInTheDocument()
  })
})
