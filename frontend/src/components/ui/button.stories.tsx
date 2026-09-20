import type { Meta, StoryObj } from '@storybook/react-vite'
import { Star } from 'lucide-react'

import { Button } from './button'

const meta = {
  title: 'ui/Button',
  component: Button,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'outline', 'secondary', 'ghost', 'destructive', 'link'],
    },
    size: {
      control: 'select',
      options: ['default', 'xs', 'sm', 'lg', 'icon', 'icon-xs', 'icon-sm', 'icon-lg'],
    },
  },
  args: {
    children: 'Button',
  },
} satisfies Meta<typeof Button>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    variant: 'default',
    size: 'default',
  },
}

export const Outline: Story = {
  args: { variant: 'outline' },
}

export const Secondary: Story = {
  args: { variant: 'secondary' },
}

export const Ghost: Story = {
  args: { variant: 'ghost' },
}

export const Destructive: Story = {
  args: { variant: 'destructive' },
}

export const Link: Story = {
  args: { variant: 'link' },
}

export const Disabled: Story = {
  args: {
    disabled: true,
  },
}

/** すべての variant を一覧表示し、パターンを網羅する */
export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Button variant="default">Default</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Destructive</Button>
      <Button variant="link">Link</Button>
    </div>
  ),
}

/** すべての size を一覧表示し、パターンを網羅する */
export const AllSizes: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Button size="xs">Extra Small</Button>
      <Button size="sm">Small</Button>
      <Button size="default">Default</Button>
      <Button size="lg">Large</Button>
      <Button size="icon-xs" aria-label="icon extra small">
        <Star />
      </Button>
      <Button size="icon-sm" aria-label="icon small">
        <Star />
      </Button>
      <Button size="icon" aria-label="icon default">
        <Star />
      </Button>
      <Button size="icon-lg" aria-label="icon large">
        <Star />
      </Button>
    </div>
  ),
}
