import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'

import { ErrorState } from './ErrorState'

const meta = {
  title: 'ui-states/ErrorState',
  component: ErrorState,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    message: 'データの取得に失敗しました',
    onRetry: fn(),
  },
} satisfies Meta<typeof ErrorState>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithoutRetry: Story = {
  args: {
    onRetry: undefined,
  },
}

export const CustomRetryLabel: Story = {
  args: {
    retryLabel: 'もう一度試す',
  },
}
