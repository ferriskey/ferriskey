import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './app.tsx'
import { BrowserRouter } from 'react-router'
import { setupStore } from './store/store.ts'
import { Provider } from 'react-redux'

const container =
  document.getElementById('root') ||
  (document.createElement('div') as HTMLElement)
const root = createRoot(container)

const store = setupStore()

const render = (
  <StrictMode>
    <Provider store={store}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </Provider>
  </StrictMode>
)

root.render(render)
