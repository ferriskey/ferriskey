import axios, { AxiosInstance } from 'axios'
import { useEffect, useMemo, useState } from 'react'
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router'
import { fetcher } from './api'
import { createApiClient } from './api/api.client'
import { TanstackQueryApiClient } from './api/api.tanstack'
import { useGetConfig } from './api/config.api'
import './App.css'
import Layout from './components/layout/layout'
import { useTheme } from './components/theme-provider'
import { Toaster } from './components/ui/sonner'
import { BasicSpinner } from './components/ui/spinner'
import { useAuth } from './hooks/use-auth'
import { useConfig } from './hooks/use-config'
import PageAuthentication from './pages/authentication/page-authentication'
import PageClientScope from './pages/client-scope/page-client-scope'
import PageClient from './pages/client/page-client'
import PageCompass from './pages/compass/page-compass'
import PageIdentityProviders from './pages/identity-providers/page-identity-providers'
import PageOverview from './pages/overview/page-overview'
import PageRealm from './pages/realm/page-realm'
import PageRole from './pages/role/page-role'
import PageSeawatch from './pages/seawatch/page-seawatch'
import PageEmailTemplate from './pages/email-template/page-email-template'
import PagePortalTheme from './pages/portal-theme/page-portal-theme'
import PageUserFederation from './pages/user-federation/page-user-federation'
import PageUser from './pages/user/page-user'
import PageOrganization from './pages/organization/page-organization'

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

function toAbsoluteApiUrl(value: string) {
  const normalized = normalizeApiUrl(value)

  try {
    return normalizeApiUrl(new URL(normalized || '/', window.location.origin).toString())
  } catch {
    return normalizeApiUrl(window.location.origin)
  }
}

function inferApiUrlCandidatesFromWindowOrigin() {
  const { hostname, origin } = window.location

  if (hostname.startsWith('accounts.')) {
    const authOrigin = origin.replace(/\/\/accounts\./, '//auth.')
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
    if (resolved) {
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

function realmFromPath(pathname: string): string {
  const match = pathname.match(/^\/realms\/([^/]+)/)
  return match?.[1] ?? 'master'
}

function App() {
  const [apiUrlSetup, setApiUrlSetup] = useState<boolean>(false)

  useEffect(() => {
    const init = async () => {
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

      const apiUrl = toAbsoluteApiUrl(uri)
      const api = createApiClient({ fetch: fetcher }).setBaseUrl(apiUrl)
      const axiosClient = axios.create({
        baseURL: apiUrl,
        headers: {
          'Content-Type': 'application/json',
        },
        withCredentials: true,
      })
      window.api = api
      window.tanstackApi = new TanstackQueryApiClient(api)
      window.apiUrl = apiUrl
      window.axios = axiosClient

      if (apiUrl) {
        setApiUrlSetup(true)
      }
    }

    void init()
  }, [])

  if (!apiUrlSetup) {
    return (
      <div className='h-screen flex items-center justify-center text-gray-500'>
        <BasicSpinner />
      </div>
    )
  }

  return <AppRoutes />
}

function AppRoutes() {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading } = useAuth()
  const { setConfig } = useConfig()
  const { theme } = useTheme()
  const { data: responseConfig } = useGetConfig()

  const currentRealm = useMemo(() => realmFromPath(pathname), [pathname])

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

    if (isLoading || pathname.includes('/authentication/callback') || (clientId && redirectUri))
      return
    if (!isAuthenticated && !authenticateRoute) {
      if (!pathname.includes('authentication/login')) {
        navigate(`/realms/${currentRealm}/authentication/login`, { replace: true })
      }
    } else if (isAuthenticated && authenticateRoute && !pathname.includes('/callback') && !pathname.includes('/required-action')) {
      navigate(`/realms/${currentRealm}/overview`, { replace: true })
    }
  }, [isAuthenticated, isLoading, authenticateRoute, pathname, currentRealm, navigate])

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
            <Route path='email-templates/*' element={<PageEmailTemplate />} />
            <Route path='portal/*' element={<PagePortalTheme />} />
            <Route path='user-federation/*' element={<PageUserFederation />} />
            <Route path='organizations/*' element={<PageOrganization />} />
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
                    ? `/realms/${currentRealm}/overview`
                    : `/realms/${currentRealm}/authentication/login`
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
                    ? `/realms/${currentRealm}/overview`
                    : `/realms/${currentRealm}/authentication/login`
                }
                replace
              />
            )
          }
        />
      </Routes>
      <Toaster richColors theme={theme as 'light' | 'dark' | 'system'} />
    </>
  )
}

export default App
