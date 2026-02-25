import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import App from './App.tsx'
import './index.css'

import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { ThemeProvider } from './components/theme-provider.tsx'

const container = document.getElementById('root') || (document.createElement('div') as HTMLElement)
const root = createRoot(container)

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false
    },
  },
})

const DISABLE_REACT_QUERY_DEVTOOLS = import.meta.env.VITE_DISABLE_REACT_QUERY_DEVTOOLS === 'true'

const render = (
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      {!DISABLE_REACT_QUERY_DEVTOOLS && (
        <ReactQueryDevtools initialIsOpen={false} />
      )}

      <BrowserRouter>
        <ThemeProvider defaultTheme='dark' storageKey='ferriskey-ui-theme'>
          <App />
        </ThemeProvider>
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>
)

root.render(render)
