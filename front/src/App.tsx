import { Route, Routes } from 'react-router'
import './app.css'
import PageAuthentication from './pages/authentication/page-authentication'

function App() {
  return (
    <>
      <Routes>
        <Route path="realms/:realm_name">
          <Route path="authentication/*" element={<PageAuthentication />} />
        </Route>
      </Routes>
    </>
  )
}

export default App
