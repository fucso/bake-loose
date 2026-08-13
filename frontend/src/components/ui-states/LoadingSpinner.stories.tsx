import type { Meta, StoryObj } from '@storybook/react-vite'

import { LoadingSpinner } from './LoadingSpinner'

const meta = {
  title: 'ui-states/LoadingSpinner',
  component: LoadingSpinner,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof LoadingSpinner>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {},
}

export const WithMessage: Story = {
  args: {
    message: '読み込み中...',
  },
}
