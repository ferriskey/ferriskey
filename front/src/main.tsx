import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Provider } from 'react-redux'
import { BrowserRouter } from 'react-router'
import App from './App.tsx'
import './index.css'
import { setupStore } from './store/store.ts'

const container = document.getElementById('root') || (document.createElement('div') as HTMLElement)
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
