import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, fn, userEvent, within } from 'storybook/test'
import { Provider } from 'urql'

import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import { CreateProjectModal } from './CreateProjectModal'

const meta = {
  title: 'projects/CreateProjectModal',
  component: CreateProjectModal,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  args: {
    open: true,
    onOpenChange: fn(),
    onCreated: fn(),
  },
  decorators: [
    (Story) => (
      <Provider value={createMockClient({})}>
        <Story />
      </Provider>
    ),
  ],
} satisfies Meta<typeof CreateProjectModal>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

/** name未入力で送信するとバリデーションエラーを表示する */
export const ValidationError: Story = {
  play: async ({ canvasElement }) => {
    // DialogPopupはPortalでdocument.body配下に描画されるため、canvasElementではなくbody全体から取得する
    const canvas = within(canvasElement.ownerDocument.body)
    await userEvent.click(canvas.getByRole('button', { name: '作成' }))
    await expect(canvas.getByText('プロジェクト名を入力してください')).toBeInTheDocument()
  },
}

/** 作成に失敗した場合、バックエンドのエラーメッセージをそのまま表示する */
export const SubmissionError: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          CreateProject: new MockGraphQLError('同じ名前のプロジェクトが既に存在します'),
        })}
      >
        <Story />
      </Provider>
    ),
  ],
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement.ownerDocument.body)
    await userEvent.type(canvas.getByLabelText('プロジェクト名'), '重複プロジェクト')
    await userEvent.click(canvas.getByRole('button', { name: '作成' }))
    await expect(
      await canvas.findByText('同じ名前のプロジェクトが既に存在します'),
    ).toBeInTheDocument()
  },
}
