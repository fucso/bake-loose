import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { StepActions } from './StepActions'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../../test/mocks/urql'
import type { Step } from '@/lib/trial'

const step: Step = {
  id: 'step-1',
  name: 'こね',
  position: 0,
  startedAt: '2026-01-01T09:00:00+09:00',
  completedAt: null,
  isCompleted: false,
  parameters: [],
}

const successResponses: MockQueryResponses = {
  UpdateStep: {
    updateStep: { id: 'step-1', name: 'こね', startedAt: '2026-01-01T09:00:00+09:00' },
  },
  CompleteStep: {
    completeStep: { id: 'step-1', isCompleted: true, completedAt: '2026-01-01T09:30:00+09:00' },
  },
}

const withClient = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <Story />
  </Provider>
)

const meta = {
  title: 'trials/StepActions',
  component: StepActions,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    trialId: 'trial-1',
    step,
    onChanged: fn(),
  },
  decorators: [withClient(successResponses)],
} satisfies Meta<typeof StepActions>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中の工程に対する編集・完了の操作 */
export const Default: Story = {}

/** 開始日時が未設定の工程。編集フォームの開始日時は空欄で開く */
export const WithoutStartedAt: Story = {
  args: {
    step: { ...step, startedAt: null },
  },
}

/** 完了に失敗した場合、操作の近くにエラーメッセージを表示する */
export const CompleteError: Story = {
  decorators: [withClient({ CompleteStep: new MockGraphQLError('既に完了している工程です') })],
}
