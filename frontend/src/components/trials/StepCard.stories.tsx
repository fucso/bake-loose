import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { StepCard } from './StepCard'
import { createMockClient } from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const step: Step = {
  id: 'step-1',
  name: '一次発酵',
  position: 1,
  startedAt: '2026-01-01T09:30:00+09:00',
  completedAt: null,
  isCompleted: false,
  parameters: [
    {
      id: 'param-1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '発酵場所',
        value: { type: 'text', value: '冷蔵庫' },
      },
    },
    {
      id: 'param-2',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
    },
    {
      id: 'param-3',
      parameterType: 'TIME_MARKER',
      content: { type: 'time_marker', at: { value: 60, unit: 'minute' }, note: '膨らみを確認' },
    },
    {
      id: 'param-4',
      parameterType: 'TEXT',
      content: { type: 'text', value: '生地がべたつく場合は打ち粉を追加' },
    },
  ],
}

const meta = {
  title: 'trials/StepCard',
  component: StepCard,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    trialId: 'trial-1',
    step,
    isCurrent: true,
    defaultExpanded: true,
    editable: true,
    onChanged: fn(),
  },
  decorators: [
    (Story) => (
      <Provider value={createMockClient({})}>
        <Story />
      </Provider>
    ),
  ],
} satisfies Meta<typeof StepCard>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中の工程。パラメーターを展開した状態で表示する */
export const Current: Story = {}

/** 完了済みの工程。一覧性を優先して畳んだ状態で表示し、記録操作は出さない */
export const CompletedCollapsed: Story = {
  args: {
    step: { ...step, isCompleted: true, completedAt: '2026-01-01T11:00:00+09:00' },
    isCurrent: false,
    defaultExpanded: false,
    editable: false,
  },
}

/** 記録できない工程（完了済みTrialなど）。パラメーターは読み取り専用で並ぶ */
export const ReadOnly: Story = {
  args: {
    isCurrent: false,
    editable: false,
  },
}

/** パラメーターがまだ記録されていない工程 */
export const WithoutParameters: Story = {
  args: {
    step: { ...step, parameters: [] },
  },
}
