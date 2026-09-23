import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn } from 'storybook/test'

import { StepFormModal } from './StepFormModal'

const meta = {
  title: 'trials/StepFormModal',
  component: StepFormModal,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    open: true,
    onOpenChange: fn(),
    title: '工程を追加',
    submitLabel: '追加',
    initialName: '',
    initialStartedAt: null,
    submitting: false,
    errorMessage: null,
    onSubmit: fn(),
  },
} satisfies Meta<typeof StepFormModal>

export default meta

type Story = StoryObj<typeof meta>

/** 工程の追加。空のフォームから入力する */
export const Add: Story = {}

/** 工程の編集。現在の工程名・開始日時が初期値として入る */
export const Edit: Story = {
  args: {
    title: '工程を編集',
    submitLabel: '保存',
    initialName: 'こね',
    initialStartedAt: '2026-01-01T09:00:00+09:00',
  },
}

/** 送信中は入力と操作を無効化する */
export const Submitting: Story = {
  args: {
    initialName: 'こね',
    submitting: true,
  },
}

/** 送信に失敗した場合、呼び出し元から渡されたメッセージを表示する */
export const WithError: Story = {
  args: {
    initialName: 'こね',
    errorMessage: '工程名は100文字以内で入力してください',
  },
}
