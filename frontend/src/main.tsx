import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { RouterProvider } from 'react-router-dom'
import { Provider } from 'urql'
import { graphqlClient } from './lib/graphql-client'
import { router } from './routes/router'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Provider value={graphqlClient}>
      <RouterProvider router={router} />
    </Provider>
  </StrictMode>,
)
