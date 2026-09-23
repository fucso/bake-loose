import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { AddStepButton } from './AddStepButton'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../../test/mocks/urql'

const addStepResponse = {
  addStep: { id: 'step-2', name: '一次発酵', position: 1, startedAt: null },
}

const withClient = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <Story />
  </Provider>
)

const meta = {
  title: 'trials/AddStepButton',
  component: AddStepButton,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    trialId: 'trial-1',
    onAdded: fn(),
  },
  decorators: [withClient({ AddStep: addStepResponse })],
} satisfies Meta<typeof AddStepButton>

export default meta

type Story = StoryObj<typeof meta>

/** 押すと工程の入力フォームが開く */
export const Default: Story = {}

/** 追加に失敗した場合、入力フォームを開いたままエラーメッセージを表示する */
export const AddError: Story = {
  decorators: [
    withClient({ AddStep: new MockGraphQLError('完了した試行には工程を追加できません') }),
  ],
}
