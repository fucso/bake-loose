import type { Meta, StoryObj } from '@storybook/react-vite'
import { MemoryRouter } from 'react-router-dom'
import { Provider } from 'urql'

import { createMockClient, MockGraphQLError } from '../../test/mocks/urql'
import ProjectsPage from './ProjectsPage'

const meta = {
  title: 'pages/ProjectsPage',
  component: ProjectsPage,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof ProjectsPage>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          Projects: {
            projects: [
              { id: '1', name: 'ピザ生地研究' },
              { id: '2', name: '食パン研究' },
            ],
          },
        })}
      >
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}

/** クエリー結果が0件の場合、空状態と作成導線が表示される */
export const Empty: Story = {
  decorators: [
    (Story) => (
      <Provider value={createMockClient({ Projects: { projects: [] } })}>
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}

/** バックエンドとの接続に失敗した場合、再試行ボタン付きのエラー状態が表示される */
export const ConnectionError: Story = {
  decorators: [
    (Story) => (
      <Provider
        value={createMockClient({
          Projects: new MockGraphQLError('接続エラーが発生しました'),
        })}
      >
        <MemoryRouter>
          <Story />
        </MemoryRouter>
      </Provider>
    ),
  ],
}
