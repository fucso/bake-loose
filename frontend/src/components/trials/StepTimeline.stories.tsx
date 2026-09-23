import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { StepTimeline } from './StepTimeline'
import { createMockClient } from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const steps: Step[] = [
  {
    id: 'step-1',
    name: 'こね',
    position: 0,
    startedAt: '2026-01-01T09:00:00+09:00',
    completedAt: '2026-01-01T09:30:00+09:00',
    isCompleted: true,
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
  },
  {
    id: 'step-2',
    name: '一次発酵',
    position: 1,
    startedAt: '2026-01-01T09:30:00+09:00',
    completedAt: null,
    isCompleted: false,
    parameters: [
      {
        id: 'param-2',
        parameterType: 'DURATION',
        content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
      },
    ],
  },
  {
    id: 'step-3',
    name: '焼成',
    position: 2,
    startedAt: null,
    completedAt: null,
    isCompleted: false,
    parameters: [],
  },
]

const meta = {
  title: 'trials/StepTimeline',
  component: StepTimeline,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    trialId: 'trial-1',
    steps,
    trialStatus: 'IN_PROGRESS',
    onChanged: fn(),
  },
  decorators: [
    (Story) => (
      <Provider value={createMockClient({})}>
        <Story />
      </Provider>
    ),
  ],
} satisfies Meta<typeof StepTimeline>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中の工程だけを展開し、完了済みの工程は畳む。末尾に工程の追加操作が並ぶ */
export const InProgress: Story = {}

/** 完了済みTrialは記録中の工程を持たず、全工程を畳んだ俯瞰表示になる。記録操作も出さない */
export const CompletedTrial: Story = {
  args: {
    steps: steps.map((step) => ({ ...step, isCompleted: true })),
    trialStatus: 'COMPLETED',
  },
}

/** 工程がまだ記録されていない状態。空状態でも追加操作は表示する */
export const Empty: Story = {
  args: {
    steps: [],
  },
}
