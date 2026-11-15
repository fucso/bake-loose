import { fireEvent, render, screen } from '@testing-library/react'
import { Provider } from 'urql'
import { describe, expect, it, vi } from 'vitest'

import { StepCard, type StepCardProps } from './StepCard'
import { createMockClient } from '../../../test/mocks/urql'
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

/** 記録操作は urql のミューテーションを使うため Provider を必要とする */
const renderCard = (props: Partial<StepCardProps> = {}) => {
  const onChanged = vi.fn()
  const utils = render(
    <Provider value={createMockClient({})}>
      <StepCard
        trialId="trial-1"
        step={buildStep()}
        isCurrent={false}
        defaultExpanded={false}
        canRecord={false}
        onChanged={onChanged}
        {...props}
      />
    </Provider>,
  )
  return { ...utils, onChanged }
}

describe('StepCard', () => {
  it('折りたたみ時もヘッダのサマリーは表示し、パラメーターは隠す', () => {
    renderCard()

    expect(screen.getByText('こね')).toBeVisible()
    expect(screen.getByText(/パラメーター 1件/)).toBeVisible()
    expect(screen.getByText('強力粉')).not.toBeVisible()
  })

  it('展開状態ではパラメーターを表示する', () => {
    renderCard({ isCurrent: true, defaultExpanded: true })

    expect(screen.getByText('強力粉')).toBeVisible()
    expect(screen.getByText('300g')).toBeVisible()
  })

  it('ヘッダのクリックで開閉を切り替えられる', () => {
    renderCard()

    const toggle = screen.getByRole('button')
    expect(toggle).toHaveAttribute('aria-expanded', 'false')

    fireEvent.click(toggle)

    expect(toggle).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByText('強力粉')).toBeVisible()
  })

  it('記録中の工程であることを示すラベルを表示する', () => {
    renderCard({ isCurrent: true, defaultExpanded: true })

    expect(screen.getByText('進行中')).toBeInTheDocument()
  })

  it('完了済みの工程は完了ラベルと完了日時を表示する', () => {
    renderCard({
      step: buildStep({ isCompleted: true, completedAt: '2026-01-01T09:30:00+09:00' }),
    })

    expect(screen.getByText('完了')).toBeInTheDocument()
    expect(screen.getByText(/完了 2026\/01\/01 09:30/)).toBeInTheDocument()
  })

  it('パラメーターが無い工程には未記録であることを表示する', () => {
    renderCard({ step: buildStep({ parameters: [] }), defaultExpanded: true })

    expect(screen.getByText('パラメーターは記録されていません')).toBeVisible()
  })

  it('記録できる工程では本文に記録操作を表示する', () => {
    renderCard({ isCurrent: true, defaultExpanded: true, canRecord: true })

    expect(screen.getByRole('button', { name: '工程を編集' })).toBeVisible()
    expect(screen.getByRole('button', { name: '工程を完了にする' })).toBeVisible()
  })

  it('記録できない工程では記録操作を表示しない', () => {
    renderCard({ defaultExpanded: true, canRecord: false })

    expect(screen.queryByRole('button', { name: '工程を編集' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '工程を完了にする' })).not.toBeInTheDocument()
  })
})
