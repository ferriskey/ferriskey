import { useCallback, useEffect, useMemo } from 'react'
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
import { fetcher } from './api'
import { TanstackQueryApiClient } from './api/api.tanstack'
import axios, { AxiosInstance } from 'axios'

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
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useAuth()
  const { setConfig } = useConfig()

  const { data: responseConfig } = useGetConfig()

  const apiCallback = useCallback(async () => {
    const viteUrl = import.meta.env.VITE_API_URL
    let uri

    if (viteUrl) {
      uri = viteUrl
    } else {
      const data = await fetch('config.json')
      const result = await data.json()
      uri = result.api_url
    }

    const api = createApiClient(fetcher, uri)
    const axiosClient = axios.create({
      baseURL: 'http://localhost:3333',
      headers: {
        'Content-Type': 'application/json',
      },
      withCredentials: true,
    })
    window.api = api
    window.tanstackApi = new TanstackQueryApiClient(api)
    window.apiUrl = uri
    window.axios = axiosClient
  }, [])

  useEffect(() => {
    apiCallback()
  }, [apiCallback])

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
    const realm = realm_name ?? 'master'

    if (!isAuthenticated && !authenticateRoute) {
      if (!pathname.includes('authentication/login')) {
        navigate(`/realms/${realm}/authentication/login`, { replace: true })
      }
    } else if (isAuthenticated && authenticateRoute && !pathname.includes('/callback')) {
      navigate(`/realms/${realm}/overview`, { replace: true })
    }
  }, [isAuthenticated, isLoading, authenticateRoute, pathname, realm_name, navigate])


  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isShortcutPressed = event.ctrlKey && event.shiftKey && event.key === 'T'

      if (isShortcutPressed) {
        const head = document.head
        let themeLink = document.getElementById('theme-overrides') as HTMLLinkElement

        if (!themeLink) {
          themeLink = document.createElement('link')
          themeLink.rel = 'stylesheet'
          themeLink.id = 'theme-overrides'
          head.appendChild(themeLink)
        }

        const currentTheme = themeLink.getAttribute('href')

        if (currentTheme === '/themes/neo-brutalism.theme.css') {
          themeLink.removeAttribute('href')
        } else {
          themeLink.href = '/themes/neo-brutalism.theme.css'
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  return (
    <>
      <Routes>
        <Route path='realms/:realm_name'>
          <Route path='authentication/*' element={<PageAuthentication />} />

          <Route element={<Layout />}>
            <Route path='overview/*' element={<PageOverview />} />

            <Route path='clients/*' element={<PageClient />} />
            <Route path='users/*' element={<PageUser />} />
            <Route path='roles/*' element={<PageRole />} />
            <Route path='realm-settings/*' element={<PageRealm />} />
          </Route>
        </Route>

        <Route path='*' element={<Navigate to='/realms/master/authentication/login' replace />} />
      </Routes>
      <Toaster
        richColors
        theme='light'
      />
    </>
  )
}

export default App
