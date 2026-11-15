import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'
import { Provider } from 'urql'

import { TrialHeader } from './TrialHeader'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../../test/mocks/urql'
import type { Trial } from '@/lib/trial'

const trial: Trial = {
  id: 'trial-1',
  projectId: 'project-1',
  name: '加水率70%',
  memo: '室温24度・冷蔵庫で一次発酵',
  status: 'IN_PROGRESS',
  completedAt: null,
  steps: [],
}

const withClient = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <Story />
  </Provider>
)

const meta = {
  title: 'trials/TrialHeader',
  component: TrialHeader,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    trial,
    onChanged: fn(),
  },
  decorators: [withClient({})],
} satisfies Meta<typeof TrialHeader>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中のTrial。編集と完了の操作が並ぶ */
export const InProgress: Story = {}

/** name が未設定の場合のフォールバック表示 */
export const WithoutName: Story = {
  args: {
    trial: { ...trial, name: null, memo: null },
  },
}

/** 完了済みのTrial。完了日時を表示し完了操作は出さない */
export const Completed: Story = {
  args: {
    trial: { ...trial, status: 'COMPLETED', completedAt: '2026-01-01T12:00:00+09:00' },
  },
}

/** 更新に失敗した場合。「編集」から保存するとエラーメッセージが出る */
export const UpdateError: Story = {
  decorators: [withClient({ UpdateTrial: new MockGraphQLError('試行名が長すぎます') })],
}
