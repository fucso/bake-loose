import { useEffect, useState } from 'react'
import { useQuery } from 'urql'

type HealthStatus = {
  status: string
  message: string
} | null

const GRAPHQL_SMOKE_TEST_QUERY = `
  query GraphqlSmokeTest {
    __typename
  }
`

function App() {
  const [health, setHealth] = useState<HealthStatus>(null)
  const [error, setError] = useState<string | null>(null)
  const [graphqlResult] = useQuery({ query: GRAPHQL_SMOKE_TEST_QUERY })

  useEffect(() => {
    const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080'
    fetch(`${apiUrl}/health`)
      .then((res) => res.json())
      .then((data) => setHealth(data))
      .catch((err) => setError(err.message))
  }, [])

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: '2rem' }}>
      <h1>bake-loose</h1>
      <p>パン・ピザ作りの試行錯誤を記録するラボノート</p>

      <h2>Backend Status</h2>
      {error && <p style={{ color: 'red' }}>Error: {error}</p>}
      {health && (
        <pre style={{ background: '#f0f0f0', padding: '1rem', borderRadius: '4px' }}>
          {JSON.stringify(health, null, 2)}
        </pre>
      )}
      {!health && !error && <p>Loading...</p>}

      <h2>GraphQL Status</h2>
      {graphqlResult.error && (
        <p style={{ color: 'red' }}>Error: {graphqlResult.error.message}</p>
      )}
      {graphqlResult.data && (
        <pre style={{ background: '#f0f0f0', padding: '1rem', borderRadius: '4px' }}>
          {JSON.stringify(graphqlResult.data, null, 2)}
        </pre>
      )}
      {graphqlResult.fetching && <p>Loading...</p>}
    </div>
  )
}

export default App
