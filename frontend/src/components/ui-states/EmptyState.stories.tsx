import type { Meta, StoryObj } from '@storybook/react-vite'
import { Pizza } from 'lucide-react'

import { Button } from '@/components/ui/button'

import { EmptyState } from './EmptyState'

const meta = {
  title: 'ui-states/EmptyState',
  component: EmptyState,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    message: 'まだ記録がありません',
  },
} satisfies Meta<typeof EmptyState>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithCustomIcon: Story = {
  args: {
    icon: Pizza,
  },
}

export const WithAction: Story = {
  args: {
    action: <Button size="sm">最初のTrialを記録する</Button>,
  },
}
