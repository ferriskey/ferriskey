import { useEffect } from 'react'
import { Route, Routes, useLocation, useNavigate, useParams } from 'react-router'
import './app.css'
import useUser from './hooks/use-user'
import PageAuthentication from './pages/authentication/page-authentication'

function App() {
  const { realm_name } = useParams()
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useUser()

  console.log(isAuthenticated, isLoading, pathname)

  useEffect(() => {
    if (!isLoading && !isAuthenticated && !pathname.includes('authentication')) {
      const realm = realm_name ?? 'master'

      navigate(`/realms/${realm}/authentication/login`)
    }
  }, [isAuthenticated, isLoading, pathname, realm_name, navigate])

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
