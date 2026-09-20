import { cacheExchange, createClient, fetchExchange } from 'urql'

export const graphqlClient = createClient({
  url: import.meta.env.VITE_GRAPHQL_ENDPOINT || 'http://localhost:8080/graphql',
  exchanges: [cacheExchange, fetchExchange],
  preferGetMethod: false,
})
