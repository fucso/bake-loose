import type { Meta, StoryObj } from '@storybook/react-vite'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { Provider } from 'urql'

import TrialDetailPage from './TrialDetailPage'
import { createMockClient, MockGraphQLError, type MockQueryResponses } from '../../test/mocks/urql'

const steps = [
  {
    id: 'step-1',
    name: 'こね',
    position: 0,
    startedAt: '2026-01-01T09:00:00+09:00',
    completedAt: '2026-01-01T09:30:00+09:00',
    isCompleted: true,
    parameters: [
      {
        id: 'param-1',
        parameterType: 'KEY_VALUE',
        content: {
          type: 'key_value',
          key: '強力粉',
          value: { type: 'quantity', amount: 300, unit: 'g' },
        },
      },
    ],
  },
  {
    id: 'step-2',
    name: '一次発酵',
    position: 1,
    startedAt: '2026-01-01T09:30:00+09:00',
    completedAt: null,
    isCompleted: false,
    parameters: [
      {
        id: 'param-2',
        parameterType: 'DURATION',
        content: { type: 'duration', duration: { value: 90, unit: 'minute' }, note: '一次発酵' },
      },
    ],
  },
]

const trial = {
  id: 'trial-1',
  projectId: 'project-1',
  name: '加水率70%',
  memo: '室温24度・冷蔵庫で一次発酵',
  status: 'IN_PROGRESS',
  completedAt: null,
  steps,
}

const withRouter = (responses: MockQueryResponses) => (Story: () => React.ReactElement) => (
  <Provider value={createMockClient(responses)}>
    <MemoryRouter initialEntries={['/projects/project-1/trials/trial-1']}>
      <Routes>
        <Route path="/projects/:id/trials/:trialId" element={<Story />} />
      </Routes>
    </MemoryRouter>
  </Provider>
)

const meta = {
  title: 'pages/TrialDetailPage',
  component: TrialDetailPage,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof TrialDetailPage>

export default meta

type Story = StoryObj<typeof meta>

/** 記録中のTrial。記録中の工程だけが展開された状態で表示される */
export const InProgress: Story = {
  decorators: [withRouter({ Trial: { trial } })],
}

/** 完了済みのTrial。全工程が畳まれ、完了日時が表示される */
export const Completed: Story = {
  decorators: [
    withRouter({
      Trial: {
        trial: {
          ...trial,
          status: 'COMPLETED',
          completedAt: '2026-01-01T12:00:00+09:00',
          steps: steps.map((step) => ({ ...step, isCompleted: true })),
        },
      },
    }),
  ],
}

/** 工程がまだ記録されていないTrial */
export const WithoutSteps: Story = {
  decorators: [withRouter({ Trial: { trial: { ...trial, steps: [] } } })],
}

/** 存在しないTrialを指定した場合 */
export const NotFound: Story = {
  decorators: [withRouter({ Trial: { trial: null } })],
}

/** 取得に失敗した場合 */
export const LoadError: Story = {
  decorators: [withRouter({ Trial: new MockGraphQLError('接続エラーが発生しました') })],
}
