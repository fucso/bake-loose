import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'

type HealthStatus = {
  status: string
  message: string
} | null

function HomePage() {
  const [health, setHealth] = useState<HealthStatus>(null)
  const [error, setError] = useState<string | null>(null)

  const checkHealth = () => {
    const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080'
    fetch(`${apiUrl}/health`)
      .then((res) => res.json())
      .then((data) => setHealth(data))
      .catch((err) => setError(err.message))
  }

  useEffect(() => {
    checkHealth()
  }, [])

  return (
    <div className="p-8 font-sans">
      <h1 className="text-2xl font-bold">bake-loose</h1>
      <p className="text-muted-foreground">パン・ピザ作りの試行錯誤を記録するラボノート</p>

      <h2 className="mt-6 text-lg font-semibold">Backend Status</h2>
      {error && <p className="text-destructive">Error: {error}</p>}
      {health && (
        <pre className="mt-2 rounded-md bg-muted p-4 text-sm">
          {JSON.stringify(health, null, 2)}
        </pre>
      )}
      {!health && !error && <p>Loading...</p>}

      <Button className="mt-4" onClick={checkHealth}>
        再確認
      </Button>
    </div>
  )
}

export default HomePage
