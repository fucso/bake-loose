import { render, screen } from '@testing-library/react'
import { Provider } from 'urql'
import { describe, expect, it, vi } from 'vitest'

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

/** 記録操作は urql のミューテーションを使うため Provider を必要とする */
const renderTimeline = (steps: Step[], trialStatus: TrialStatus = 'IN_PROGRESS') =>
  render(
    <Provider value={createMockClient({})}>
      <StepTimeline
        trialId="trial-1"
        steps={steps}
        trialStatus={trialStatus}
        onChanged={vi.fn()}
      />
    </Provider>,
  )

describe('StepTimeline', () => {
  it('工程が無い場合は空状態を表示する', () => {
    renderTimeline([])

    expect(screen.getByText('まだ工程が記録されていません')).toBeInTheDocument()
  })

  it('position昇順で表示する', () => {
    renderTimeline([
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
      buildStep({ id: 'a', position: 0, name: 'こね' }),
    ])

    const items = screen.getAllByRole('listitem')
    expect(items[0]).toHaveTextContent('こね')
    expect(items[1]).toHaveTextContent('一次発酵')
  })

  it('最初の未完了工程を記録中の工程として示す', () => {
    renderTimeline([
      buildStep({ id: 'a', position: 0, name: 'こね', isCompleted: true }),
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
      buildStep({ id: 'c', position: 2, name: '焼成' }),
    ])

    const items = screen.getAllByRole('listitem')
    expect(items[1]).toHaveAttribute('aria-current', 'step')
    expect(items[0]).not.toHaveAttribute('aria-current')
    expect(items[2]).not.toHaveAttribute('aria-current')
  })

  it('記録中の工程は展開し、完了済みの工程は畳む', () => {
    renderTimeline([
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
    ])

    expect(screen.getByText('完了済みメモ')).not.toBeVisible()
    expect(screen.getByText('記録中メモ')).toBeVisible()
  })

  it('完了済みTrialでは記録中の工程を持たず、全工程を畳んで表示する', () => {
    renderTimeline(
      [
        buildStep({
          id: 'a',
          position: 0,
          name: 'こね',
          parameters: [
            { id: 'p1', parameterType: 'TEXT', content: { type: 'text', value: 'メモ' } },
          ],
        }),
      ],
      'COMPLETED',
    )

    expect(screen.getByRole('listitem')).not.toHaveAttribute('aria-current')
    expect(screen.getByText('メモ')).not.toBeVisible()
  })

  it('記録中のTrialでは工程が無くても追加操作を表示する', () => {
    renderTimeline([])

    expect(screen.getByRole('button', { name: '+ 工程を追加' })).toBeInTheDocument()
  })

  it('未完了の工程にだけ記録操作を表示する', () => {
    renderTimeline([
      buildStep({ id: 'a', position: 0, name: 'こね', isCompleted: true }),
      buildStep({ id: 'b', position: 1, name: '一次発酵' }),
    ])

    // 完了済みの工程は編集・完了ができないため、記録操作は記録中の工程の分だけになる
    expect(screen.getAllByRole('button', { name: '工程を編集' })).toHaveLength(1)
    expect(screen.getAllByRole('button', { name: '工程を完了にする' })).toHaveLength(1)
  })

  it('完了済みTrialでは工程の追加・記録操作を表示しない', () => {
    renderTimeline([buildStep({ id: 'a', position: 0, name: 'こね' })], 'COMPLETED')

    expect(screen.queryByRole('button', { name: '+ 工程を追加' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '工程を編集' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '工程を完了にする' })).not.toBeInTheDocument()
  })
})
