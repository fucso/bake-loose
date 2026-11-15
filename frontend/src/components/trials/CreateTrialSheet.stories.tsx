import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, fn, userEvent, within } from 'storybook/test'
import { Provider } from 'urql'

import { createMockClient, MockGraphQLError } from '../../../test/mocks/urql'
import { CreateTrialSheet } from './CreateTrialSheet'

const meta = {
  title: 'trials/CreateTrialSheet',
  component: CreateTrialSheet,
  parameters: {
    layout: 'fullscreen',
  },
  tags: ['autodocs'],
  args: {
    projectId: 'project-1',
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
} satisfies Meta<typeof CreateTrialSheet>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

/** name / memo はいずれも任意のため、未入力のままでも作成できる */
export const EmptySubmission: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          CreateTrial: { createTrial: { id: 'trial-1' } },
        })}
      >
        <Story />
      </Provider>
    ),
  ],
  play: async ({ args, canvasElement }) => {
    // DialogPopupはPortalでdocument.body配下に描画されるため、canvasElementではなくbody全体から取得する
    const canvas = within(canvasElement.ownerDocument.body)
    await userEvent.click(canvas.getByRole('button', { name: '作成' }))
    await expect(args.onCreated).toHaveBeenCalled()
  },
}

/** 作成に失敗した場合、バックエンドのエラーメッセージをそのまま表示する */
export const SubmissionError: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          CreateTrial: new MockGraphQLError('指定されたプロジェクトが見つかりません'),
        })}
      >
        <Story />
      </Provider>
    ),
  ],
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement.ownerDocument.body)
    await userEvent.type(canvas.getByLabelText('試行名（任意）'), '加水率70%')
    await userEvent.click(canvas.getByRole('button', { name: '作成' }))
    await expect(
      await canvas.findByText('指定されたプロジェクトが見つかりません'),
    ).toBeInTheDocument()
  },
}
