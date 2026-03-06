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
import PageCompass from './pages/compass/page-compass'
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

function normalizeApiUrl(value: string) {
  return value.trim().replace(/\/+$/, '')
}

function inferApiUrlCandidatesFromWindowOrigin() {
  const { hostname, origin, protocol } = window.location

  if (hostname.startsWith('accounts.')) {
    const authOrigin = `${protocol}//auth.${hostname.slice('accounts.'.length)}`
    return [`${authOrigin}/api`, authOrigin]
  }

  if (hostname.startsWith('auth.')) {
    return [`${origin}/api`, origin]
  }

  return [origin]
}

function withApiPathVariant(url: string) {
  try {
    const parsed = new URL(url, window.location.origin)
    const currentPath = parsed.pathname.replace(/\/+$/, '')

    if (!currentPath) {
      parsed.pathname = '/api'
      return normalizeApiUrl(parsed.toString())
    }

    if (currentPath === '/api') {
      parsed.pathname = ''
      return normalizeApiUrl(parsed.toString())
    }
  } catch {
    // Fallback to base url if URL parsing fails.
  }

  return null
}

async function isReachableApiBase(url: string) {
  const controller = new AbortController()
  const timeoutId = window.setTimeout(() => controller.abort(), 1500)

  try {
    const response = await fetch(`${normalizeApiUrl(url)}/health/live`, {
      method: 'GET',
      credentials: 'include',
      signal: controller.signal,
    })

    return response.ok
  } catch {
    return false
  } finally {
    window.clearTimeout(timeoutId)
  }
}

async function resolveApiUrl(baseUrl: string) {
  const primary = normalizeApiUrl(baseUrl)
  const fallback = withApiPathVariant(primary)
  const candidates = [primary, fallback].filter((value, index, self): value is string => {
    return typeof value === 'string' && value.length > 0 && self.indexOf(value) === index
  })

  for (const candidate of candidates) {
    if (await isReachableApiBase(candidate)) {
      return candidate
    }
  }

  return primary
}

async function resolveApiUrlFromWindowOrigin() {
  const candidates = inferApiUrlCandidatesFromWindowOrigin()

  for (const candidate of candidates) {
    const resolved = await resolveApiUrl(candidate)
    if (await isReachableApiBase(resolved)) {
      return resolved
    }
  }

  return normalizeApiUrl(candidates[0] ?? window.location.origin)
}

function isUnresolvedApiUrl(value: unknown) {
  if (typeof value !== 'string') {
    return true
  }

  const trimmed = value.trim()
  return trimmed.length === 0 || trimmed.includes('${')
}

function App() {
  const { realm_name } = useParams()
  const [apiUrlSetup, setApiUrlSetup] = useState<boolean>(false)
  const defaultRealm = realm_name ?? 'master'

  const apiCallback = useCallback(async () => {
    let uri = import.meta.env.VITE_API_URL?.trim()

    if (isUnresolvedApiUrl(uri)) {
      try {
        const data = await fetch('/config.json')
        const result = await data.json()
        uri = typeof result?.api_url === 'string' ? result.api_url.trim() : ''
      } catch {
        uri = ''
      }
    }

    if (isUnresolvedApiUrl(uri)) {
      uri = await resolveApiUrlFromWindowOrigin()
    } else {
      uri = await resolveApiUrl(uri)
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
    // Initialization updates app-level clients and marks setup completion.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void apiCallback()
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
            <Route path='compass/*' element={<PageCompass />} />
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
