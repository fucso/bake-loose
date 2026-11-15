import type { Meta, StoryObj } from '@storybook/react-vite'

import { Input } from './input'

const meta = {
  title: 'ui/Input',
  component: Input,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  args: {
    placeholder: '入力してください',
  },
} satisfies Meta<typeof Input>

export default meta

type Story = StoryObj<typeof meta>

/** 標準のテキスト入力 */
export const Default: Story = {}

/** 値が入っている状態 */
export const Filled: Story = {
  args: {
    defaultValue: '強力粉',
  },
}

/** 送信中など操作を受け付けない状態 */
export const Disabled: Story = {
  args: {
    defaultValue: '強力粉',
    disabled: true,
  },
}

/** 数値入力。モバイルで数字キーパッドが出るよう inputMode を指定する */
export const Decimal: Story = {
  args: {
    inputMode: 'decimal',
    defaultValue: '300',
  },
}
