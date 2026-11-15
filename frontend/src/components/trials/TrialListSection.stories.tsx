import type { Meta, StoryObj } from '@storybook/react-vite'
import { MemoryRouter } from 'react-router-dom'
import { Provider } from 'urql'

import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import { TrialListSection } from './TrialListSection'

const meta = {
  title: 'trials/TrialListSection',
  component: TrialListSection,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    projectId: 'project-1',
  },
} satisfies Meta<typeof TrialListSection>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          TrialsByProject: {
            trialsByProject: [
              { id: 'trial-1', name: '加水率70%', status: 'IN_PROGRESS', completedAt: null },
              {
                id: 'trial-2',
                name: '加水率65% / 低温長時間発酵',
                status: 'COMPLETED',
                completedAt: '2026-03-01T09:30:00+09:00',
              },
              { id: 'trial-3', name: null, status: 'IN_PROGRESS', completedAt: null },
            ],
          },
        })}
      >
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}

/** Trial が0件の場合、空状態と作成導線が表示される */
export const Empty: Story = {
  decorators: [
    (Story) => (
      <Provider value={createMockClient({ TrialsByProject: { trialsByProject: [] } })}>
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}

/** 取得に失敗した場合、再試行ボタン付きのエラー状態が表示される */
export const ConnectionError: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          TrialsByProject: new MockGraphQLError('接続エラーが発生しました'),
        })}
      >
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}
