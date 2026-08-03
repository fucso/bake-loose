import { Link, Outlet } from 'react-router-dom'

function RootLayout() {
  return (
    <div className="font-sans">
      <header className="border-b px-8 py-4">
        <Link to="/" className="text-xl font-bold">
          bake-loose
        </Link>
      </header>
      <main className="p-8">
        <Outlet />
      </main>
    </div>
  )
}

export default RootLayout
