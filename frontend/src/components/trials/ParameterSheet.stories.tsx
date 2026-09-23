import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { ParameterSheet } from './ParameterSheet'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../../test/mocks/urql'
import type { Parameter } from '@/lib/trial'

const withClient = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <Story />
  </Provider>
)

const quantityParameter: Parameter = {
  id: 'param-1',
  parameterType: 'KEY_VALUE',
  content: {
    type: 'key_value',
    key: '強力粉',
    value: { type: 'quantity', amount: 300, unit: 'g' },
  },
}

const durationParameter: Parameter = {
  id: 'param-2',
  parameterType: 'DURATION',
  content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
}

const meta = {
  title: 'trials/ParameterSheet',
  component: ParameterSheet,
  parameters: {
    layout: 'fullscreen',
  },
  tags: ['autodocs'],
  args: {
    open: true,
    onOpenChange: fn(),
    trialId: 'trial-1',
    stepId: 'step-1',
    onSaved: fn(),
  },
  decorators: [withClient({})],
} satisfies Meta<typeof ParameterSheet>

export default meta

type Story = StoryObj<typeof meta>

/** 新規追加。種別を選んでから種別ごとのフォームを入力する */
export const Add: Story = {}

/** 数量を持つ項目の編集。種別と値の種類は固定される */
export const EditQuantity: Story = {
  args: {
    parameter: quantityParameter,
  },
}

/** 経過時間の編集 */
export const EditDuration: Story = {
  args: {
    parameter: durationParameter,
  },
}

/** 追加に失敗した場合。バックエンドのエラーメッセージをそのまま表示する */
export const AddError: Story = {
  decorators: [withClient({ AddParameter: new MockGraphQLError('工程は完了済みです') })],
}
