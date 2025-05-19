import { useEffect, useMemo } from 'react'
import { Route, Routes, useLocation, useNavigate, useParams } from 'react-router'
import './App.css'
import Layout from './components/layout/layout'
import PageAuthentication from './pages/authentication/page-authentication'
import PageOverview from './pages/overview/page-overview'
import { useAuth } from './hooks/use-auth'
import PageClient from './pages/client/page-client'

function App() {
  const { realm_name } = useParams()
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useAuth()

  const authenticateRoute = useMemo(() => {
    if (pathname.includes('authentication')) {
      return true
    }
    return false
  }, [pathname])

  useEffect(() => {
    if (isLoading || pathname.includes('/authentication/callback')) return
    const realm = realm_name ?? 'master'
    
    if (!isAuthenticated && !authenticateRoute) {
      if (!pathname.includes('authentication/login')) {
        navigate(`/realms/${realm}/authentication/login`, { replace: true });
      }
    } else if (isAuthenticated && authenticateRoute && !pathname.includes('/callback')) {
      navigate(`/realms/${realm}/overview`, { replace: true });
    }
  }, [isAuthenticated, isLoading, authenticateRoute, pathname, realm_name])

  return (
    <>
      <Routes>
        <Route path="realms/:realm_name">
          <Route path="authentication/*" element={<PageAuthentication />} />

          <Route element={<Layout />}>
            <Route path="overview/*" element={<PageOverview />} />

            <Route path='clients/*' element={<PageClient />} />
          </Route>
        </Route>
      </Routes>
    </>
  )
}

export default App
