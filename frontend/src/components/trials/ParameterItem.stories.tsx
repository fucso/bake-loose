import type { Meta, StoryObj } from '@storybook/react-vite'

import { ParameterItem } from './ParameterItem'

const meta = {
  title: 'trials/ParameterItem',
  component: ParameterItem,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <ul>
        <Story />
      </ul>
    ),
  ],
  args: {
    parameter: {
      id: 'param-1',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '強力粉',
        value: { type: 'quantity', amount: 300, unit: 'g' },
      },
    },
  },
} satisfies Meta<typeof ParameterItem>

export default meta

type Story = StoryObj<typeof meta>

/** キーと数量の組 */
export const KeyValueQuantity: Story = {}

/** キーとテキストの組 */
export const KeyValueText: Story = {
  args: {
    parameter: {
      id: 'param-2',
      parameterType: 'KEY_VALUE',
      content: {
        type: 'key_value',
        key: '発酵場所',
        value: { type: 'text', value: '冷蔵庫' },
      },
    },
  },
}

/** 経過時間 */
export const Duration: Story = {
  args: {
    parameter: {
      id: 'param-3',
      parameterType: 'DURATION',
      content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
    },
  },
}

/** 時間マーカー */
export const TimeMarker: Story = {
  args: {
    parameter: {
      id: 'param-4',
      parameterType: 'TIME_MARKER',
      content: { type: 'time_marker', at: { value: 30, unit: 'minute' }, note: '焼成開始から' },
    },
  },
}

/** 自由記述。ラベルを持たない */
export const Text: Story = {
  args: {
    parameter: {
      id: 'param-5',
      parameterType: 'TEXT',
      content: { type: 'text', value: '生地がべたつく場合は打ち粉を追加' },
    },
  },
}
