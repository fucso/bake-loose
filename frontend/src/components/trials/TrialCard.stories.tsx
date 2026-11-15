import type { Meta, StoryObj } from '@storybook/react-vite'
import { MemoryRouter } from 'react-router-dom'

import { TrialCard } from './TrialCard'

const meta = {
  title: 'trials/TrialCard',
  component: TrialCard,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    projectId: 'project-1',
    trial: {
      id: 'trial-1',
      name: '加水率70%',
      status: 'IN_PROGRESS',
      completedAt: null,
    },
  },
  decorators: [
    (Story) => (
      <MemoryRouter>
        <div className="w-80">
          <Story />
        </div>
      </MemoryRouter>
    ),
  ],
} satisfies Meta<typeof TrialCard>

export default meta

type Story = StoryObj<typeof meta>

export const InProgress: Story = {}

/** 完了済みの Trial は完了バッジと完了日時を表示する */
export const Completed: Story = {
  args: {
    trial: {
      id: 'trial-2',
      name: '加水率65% / 低温長時間発酵',
      status: 'COMPLETED',
      completedAt: '2026-03-01T09:30:00+09:00',
    },
  },
}

/** name 未設定の Trial はフォールバックラベルを表示する */
export const Unnamed: Story = {
  args: {
    trial: {
      id: 'trial-3',
      name: null,
      status: 'IN_PROGRESS',
      completedAt: null,
    },
  },
}
