import { Link, Outlet } from 'react-router-dom'

function RootLayout() {
  return (
    <div style={{ fontFamily: 'system-ui, sans-serif' }}>
      <header style={{ padding: '1rem 2rem', borderBottom: '1px solid #ddd' }}>
        <h1 style={{ margin: 0 }}>bake-loose</h1>
        <nav style={{ marginTop: '0.5rem' }}>
          <Link to="/" style={{ marginRight: '1rem' }}>
            Home
          </Link>
          <Link to="/projects">Projects</Link>
        </nav>
      </header>
      <main style={{ padding: '2rem' }}>
        <Outlet />
      </main>
    </div>
  )
}

export default RootLayout
