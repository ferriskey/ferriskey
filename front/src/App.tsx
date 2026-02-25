import { useCallback, useEffect, useMemo, useState } from 'react'
import { Navigate, Route, Routes, useLocation, useNavigate, useParams } from 'react-router'
import './App.css'
import Layout from './components/layout/layout'
import { useAuth } from './hooks/use-auth'
import PageAuthentication from './pages/authentication/page-authentication'
import PageClient from './pages/client/page-client'
import PageOverview from './pages/overview/page-overview'
import PageRole from './pages/role/page-role'
import PageUser from './pages/user/page-user'
import PageRealm from './pages/realm/page-realm'
import { Toaster } from './components/ui/sonner'
import { useGetConfig } from './api/config.api'
import { useConfig } from './hooks/use-config'
import { createApiClient } from './api/api.client'
import { TanstackQueryApiClient } from './api/api.tanstack'
import axios, { AxiosInstance } from 'axios'
import { BasicSpinner } from './components/ui/spinner'
import { fetcher } from './api'
import PageSeawatch from './pages/seawatch/page-seawatch'
import PageIdentityProviders from './pages/identity-providers/page-identity-providers'
import { useTheme } from './components/theme-provider'
import PageUserFederation from './pages/user-federation/page-user-federation'
import PageClientScope from './pages/client-scope/page-client-scope'

declare global {
  interface Window {
    api: ReturnType<typeof createApiClient>
    tanstackApi: TanstackQueryApiClient
    apiUrl: string
    axios: AxiosInstance
  }
}

function App() {
  const { realm_name } = useParams()
  const [apiUrlSetup, setApiUrlSetup] = useState<boolean>(false)
  const defaultRealm = realm_name ?? 'master'

  const apiCallback = useCallback(async () => {
    const viteUrl = import.meta.env.VITE_API_URL
    let uri

    if (viteUrl) {
      uri = viteUrl
    } else {
      const data = await fetch('/config.json')
      const result = await data.json()
      uri = result.api_url
    }

    const api = createApiClient(fetcher, uri)
    const axiosClient = axios.create({
      baseURL: uri,
      headers: {
        'Content-Type': 'application/json',
      },
      withCredentials: true,
    })
    window.api = api
    window.tanstackApi = new TanstackQueryApiClient(api)
    window.apiUrl = uri
    window.axios = axiosClient

    if (typeof uri === 'string' && uri) {
      setApiUrlSetup(true)
    }
  }, [])

  useEffect(() => {
    apiCallback()
  }, [apiCallback])

  if (!apiUrlSetup) {
    return (
      <div className='h-screen flex items-center justify-center text-gray-500'>
        <BasicSpinner />
      </div>
    )
  }

  return <AppRoutes defaultRealm={defaultRealm} />
}

function AppRoutes({ defaultRealm }: { defaultRealm: string }) {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useAuth()
  const { setConfig } = useConfig()
  const { theme } = useTheme()
  const { data: responseConfig } = useGetConfig()

  useEffect(() => {
    if (responseConfig) {
      setConfig(responseConfig)
    }
  }, [responseConfig, setConfig])

  const authenticateRoute = useMemo(() => {
    if (pathname.includes('authentication')) {
      return true
    }
    return false
  }, [pathname])

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search)
    const clientId = urlParams.get('client_id')
    const redirectUri = urlParams.get('redirect_uri')

    if (isLoading || pathname.includes('/authentication/callback') || (clientId && redirectUri)) return
    if (!isAuthenticated && !authenticateRoute) {
      if (!pathname.includes('authentication/login')) {
        navigate(`/realms/${defaultRealm}/authentication/login`, { replace: true })
      }
    } else if (isAuthenticated && authenticateRoute && !pathname.includes('/callback')) {
      navigate(`/realms/${defaultRealm}/overview`, { replace: true })
    }
  }, [isAuthenticated, isLoading, authenticateRoute, pathname, defaultRealm, navigate])

  return (
    <>
      <Routes>
        <Route path='realms/:realm_name'>
          <Route path='authentication/*' element={<PageAuthentication />} />

          <Route element={<Layout />}>
            <Route path='overview/*' element={<PageOverview />} />

            <Route path='clients/*' element={<PageClient />} />
            <Route path='client-scopes/*' element={<PageClientScope />} />
            <Route path='users/*' element={<PageUser />} />
            <Route path='roles/*' element={<PageRole />} />
            <Route path='realm-settings/*' element={<PageRealm />} />
            <Route path='seawatch/*' element={<PageSeawatch />} />
            <Route path='identity-providers/*' element={<PageIdentityProviders />} />
            <Route path='user-federation/*' element={<PageUserFederation />} />
          </Route>
        </Route>

        <Route
          path='/'
          element={
            isLoading ? (
              <div className='h-screen flex items-center justify-center text-gray-500'>
                <BasicSpinner />
              </div>
            ) : (
              <Navigate
                to={
                  isAuthenticated
                    ? `/realms/${defaultRealm}/overview`
                    : `/realms/${defaultRealm}/authentication/login`
                }
                replace
              />
            )
          }
        />
        <Route
          path='*'
          element={
            isLoading ? (
              <div className='h-screen flex items-center justify-center text-gray-500'>
                <BasicSpinner />
              </div>
            ) : (
              <Navigate
                to={
                  isAuthenticated
                    ? `/realms/${defaultRealm}/overview`
                    : `/realms/${defaultRealm}/authentication/login`
                }
                replace
              />
            )
          }
        />
      </Routes>
      <Toaster
        richColors
        theme={theme as 'light' | 'dark' | 'system'}
      />
    </>
  )
}

export default App
