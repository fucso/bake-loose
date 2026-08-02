import { render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { Provider } from 'urql'
import { createMockClient } from '@/mocks/urql'
import HomePage from './HomePage'

describe('HomePage', () => {
  it('GraphQLの疎通確認クエリのモック応答を画面に表示する', async () => {
    const mockClient = createMockClient({
      GraphqlSmokeTest: { __typename: 'Query' },
    })

    // Backend Status のヘルスチェックは本テストの対象外のため、最小限にスタブしておく
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        json: () => Promise.resolve({ status: 'ok', message: 'stub' }),
      }),
    )

    render(
      <Provider value={mockClient}>
        <HomePage />
      </Provider>,
    )

    await waitFor(() => {
      expect(screen.getByText(/"__typename": "Query"/)).toBeInTheDocument()
    })
  })
})
