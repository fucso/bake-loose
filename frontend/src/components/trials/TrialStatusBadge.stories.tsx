import type { Meta, StoryObj } from '@storybook/react-vite'

import { TrialStatusBadge } from './TrialStatusBadge'

const meta = {
  title: 'trials/TrialStatusBadge',
  component: TrialStatusBadge,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    status: 'IN_PROGRESS',
  },
} satisfies Meta<typeof TrialStatusBadge>

export default meta

type Story = StoryObj<typeof meta>

export const InProgress: Story = {}

export const Completed: Story = {
  args: {
    status: 'COMPLETED',
  },
}
