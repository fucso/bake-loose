import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { ParameterItem } from './ParameterItem'
import type { Parameter } from '@/lib/trial'

const renderItem = (parameter: Parameter) =>
  render(
    <ul>
      <ParameterItem parameter={parameter} />
    </ul>,
  )

describe('ParameterItem', () => {
  it('ラベルを持つパラメーターはラベルと値を表示する', () => {
    renderItem({
      id: '1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    })

    expect(screen.getByText('強力粉')).toBeInTheDocument()
    expect(screen.getByText('300g')).toBeInTheDocument()
  })

  it('自由記述はラベルを持たず値のみ表示する', () => {
    const { container } = renderItem({
      id: '2',
      parameterType: 'TEXT',
      content: { type: 'text', value: '打ち粉を追加' },
    })

    expect(screen.getByText('打ち粉を追加')).toBeInTheDocument()
    expect(container.querySelectorAll('span')).toHaveLength(1)
  })
})
