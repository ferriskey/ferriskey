import { useLocation, useNavigate, useParams } from 'react-router'
import useUiModeStore, { UiMode } from '@/store/ui-mode.store'
import { RouterParams } from '@/routes/router'

const consoleDefault = (realm: string) => `/realms/${realm}/console/activity/live`
const adminDefault = (realm: string) => `/realms/${realm}/overview`

/** Derives the current UI mode strictly from the URL. */
export function deriveModeFromPath(pathname: string): UiMode {
  return pathname.includes('/console/') || pathname.endsWith('/console') ? 'console' : 'admin'
}

/**
 * Maps an admin path to its console equivalent (and vice-versa) when an
 * explicit pairing exists. Returns null if no specific mapping is defined —
 * callers fall back to last-visited or the mode default.
 */
function explicitMap(pathname: string, toMode: UiMode, realm: string): string | null {
  const root = `/realms/${realm}`
  const after = pathname.startsWith(root) ? pathname.slice(root.length) : pathname

  if (toMode === 'console') {
    if (after === '/overview' || after.startsWith('/overview/')) return `${root}/console/activity/live`
    if (after === '/users' || after === '/users/overview') {
      return `${root}/console/user-management/identities`
    }
    if (after === '/organizations') return `${root}/console/user-management/organizations`
    if (after === '/roles' || after === '/roles/overview') return `${root}/console/user-management/roles`
    if (after.startsWith('/clients')) return `${root}/console/applications`
    if (after.startsWith('/compass') || after.startsWith('/seawatch')) {
      return `${root}/console/activity/logs`
    }
    if (after.startsWith('/identity-providers')) {
      return `${root}/console/authentication/identity-providers`
    }
    if (after.startsWith('/user-federation')) {
      return `${root}/console/authentication/user-federation`
    }
    if (after.startsWith('/realm-settings')) {
      return `${root}/console/authentication/sign-in-methods`
    }
    if (after.startsWith('/email-templates')) return `${root}/console/branding/email-templates`
    return null
  }

  if (after.startsWith('/console/activity/live')) return `${root}/overview`
  if (after.startsWith('/console/activity/logs')) return `${root}/seawatch/overview`
  if (after.startsWith('/console/user-management/identities')) return `${root}/users/overview`
  if (after.startsWith('/console/user-management/organizations')) return `${root}/organizations`
  if (after.startsWith('/console/user-management/roles')) return `${root}/roles/overview`
  if (after.startsWith('/console/applications')) return `${root}/clients/overview`
  if (after.startsWith('/console/authentication/identity-providers')) {
    return `${root}/identity-providers`
  }
  if (after.startsWith('/console/authentication/user-federation')) return `${root}/user-federation`
  if (after.startsWith('/console/authentication')) return `${root}/realm-settings`
  if (after.startsWith('/console/branding')) return `${root}/email-templates`
  return null
}

export function useSwitchMode() {
  const lastVisited = useUiModeStore((s) => s.lastVisited)
  const setLastVisited = useUiModeStore((s) => s.setLastVisited)
  const navigate = useNavigate()
  const location = useLocation()
  const { realm_name } = useParams<RouterParams>()

  return (newMode: UiMode) => {
    const realm = realm_name ?? 'master'
    const currentMode = deriveModeFromPath(location.pathname)
    if (newMode === currentMode) return

    setLastVisited(currentMode, location.pathname)

    const remembered = lastVisited[newMode]
    const fallbackDefault = newMode === 'console' ? consoleDefault(realm) : adminDefault(realm)
    const target =
      remembered ?? explicitMap(location.pathname, newMode, realm) ?? fallbackDefault

    if (target !== location.pathname) navigate(target, { replace: true })
  }
}
