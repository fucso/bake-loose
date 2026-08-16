import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'

import { cn } from '@/lib/utils'

import { Button, buttonVariants } from './button'
import { Dialog, DialogClose, DialogPopup, DialogTitle } from './dialog'

const meta = {
  title: 'ui/Dialog',
  component: Dialog,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof Dialog>

export default meta

type Story = StoryObj<typeof meta>

function DialogDemo() {
  const [open, setOpen] = useState(false)

  return (
    <>
      <Button onClick={() => setOpen(true)}>ダイアログを開く</Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogPopup>
          <DialogTitle>ダイアログのタイトル</DialogTitle>
          <p className="mt-2 text-sm text-muted-foreground">
            ダイアログの本文がここに入ります。背景クリックまたはEscキーでも閉じます。
          </p>
          <div className="mt-4 flex justify-end gap-2">
            <DialogClose className={cn(buttonVariants({ variant: 'outline' }))}>
              閉じる
            </DialogClose>
          </div>
        </DialogPopup>
      </Dialog>
    </>
  )
}

/** トリガーボタンから開閉できる、実際の使用に近い状態 */
export const Default: Story = {
  render: () => <DialogDemo />,
}

/** 常に開いた状態で表示し、見た目・レイアウトを確認できる */
export const Open: Story = {
  render: () => (
    <Dialog open>
      <DialogPopup>
        <DialogTitle>常に開いた状態のダイアログ</DialogTitle>
        <p className="mt-2 text-sm text-muted-foreground">
          Storybook上で常時表示された状態を確認できます。
        </p>
      </DialogPopup>
    </Dialog>
  ),
}
