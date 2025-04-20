import { useEffect, useMemo } from 'react'
import { Route, Routes, useLocation, useNavigate, useParams } from 'react-router'
import './app.css'
import Layout from './components/layout/layout'
import useUser from './hooks/use-user'
import PageAuthentication from './pages/authentication/page-authentication'
import PageOverview from './pages/overview/page-overview'

function App() {
  const { realm_name } = useParams()
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useUser()

  console.log(isAuthenticated, isLoading, pathname)

  const authenticateRoute = useMemo(() => {
    if (pathname.includes('authentication')) {
      return true
    }
    return false
  }, [pathname])

  useEffect(() => {
    const realm = realm_name ?? 'master'
    if (!isLoading && !isAuthenticated && !authenticateRoute) {
      navigate(`/realms/${realm}/authentication/login`)
    } else {
      if (isAuthenticated) {
        navigate(`/realms/${realm}/overview`)
      }
    }
  }, [isAuthenticated, isLoading, pathname, realm_name, navigate])

  return (
    <>
      <Routes>
        <Route path="realms/:realm_name">
          <Route path="authentication/*" element={<PageAuthentication />} />

          <Route element={<Layout />}>
            <Route path="overview/*" element={<PageOverview />} />
          </Route>
        </Route>
      </Routes>
    </>
  )
}

export default App
