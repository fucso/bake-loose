import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { StepParameterList } from './StepParameterList'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const withClient = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <Story />
  </Provider>
)

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
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    },
    {
      id: 'param-2',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
    },
    {
      id: 'param-3',
      parameterType: 'TEXT',
      content: { type: 'text', value: '生地がべたつく場合は打ち粉を追加' },
    },
  ],
}

const meta = {
  title: 'trials/StepParameterList',
  component: StepParameterList,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    trialId: 'trial-1',
    step,
    editable: true,
    onChanged: fn(),
  },
  decorators: [withClient({})],
} satisfies Meta<typeof StepParameterList>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中の工程。各パラメーターに編集・削除、末尾に追加の操作が並ぶ */
export const Editable: Story = {}

/** 完了済みの工程・Trial。読み取り専用で操作は出さない */
export const ReadOnly: Story = {
  args: {
    editable: false,
  },
}

/** パラメーターがまだ記録されていない工程 */
export const Empty: Story = {
  args: {
    step: { ...step, parameters: [] },
  },
}

/** 削除に失敗した場合。削除を確認して実行するとエラーメッセージが出る */
export const RemoveError: Story = {
  decorators: [withClient({ RemoveParameter: new MockGraphQLError('試行は完了済みです') })],
}
