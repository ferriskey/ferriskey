import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { REALM_URL } from '@/routes/router'
import { CONSOLE_URL } from '@/routes/router'

export type Panel = 'admin' | 'console'

const CONSOLE_DEFAULT = 'activity/live'
const ADMIN_DEFAULT = 'overview'

const PAIRS: Array<[admin: string, section: string]> = [
  ['overview', CONSOLE_DEFAULT],
  ['users', 'user-management/identities'],
  ['organizations', 'user-management/organizations'],
  ['roles', 'user-management/roles'],
  ['clients', 'applications'],
  ['seawatch', 'activity/logs'],
  ['compass', 'activity/logs'],
  ['identity-providers', 'authentication/identity-providers'],
  ['realm-settings', 'authentication/sign-in-methods'],
  ['email-templates', 'branding/email-templates'],
  ['portal', 'branding/themes'],
]

export function usePanel(): Panel {
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()
  return pathname.startsWith(CONSOLE_URL(realm_name)) ? 'console' : 'admin'
}

export function useSwitchPanel() {
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()

  return (target: Panel) => {
    const adminRoot = REALM_URL(realm_name)
    const consoleRoot = CONSOLE_URL(realm_name)
    const current: Panel = pathname.startsWith(consoleRoot) ? 'console' : 'admin'
    if (current === target) return

    if (target === 'console') {
      const rest = pathname.startsWith(adminRoot) ? pathname.slice(adminRoot.length + 1) : ''
      const pair = PAIRS.find(([admin]) => rest === admin || rest.startsWith(`${admin}/`))
      navigate(`${consoleRoot}/${pair ? pair[1] : CONSOLE_DEFAULT}`, { replace: true })
      return
    }

    const rest = pathname.startsWith(consoleRoot) ? pathname.slice(consoleRoot.length + 1) : ''
    const pair = PAIRS.find(([, section]) => rest === section || rest.startsWith(`${section}/`))
    navigate(`${adminRoot}/${pair ? pair[0] : ADMIN_DEFAULT}`, { replace: true })
  }
}
