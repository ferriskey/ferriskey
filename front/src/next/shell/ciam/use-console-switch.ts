import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { NEXT_URL } from '@/next/routes'
import { NEXT_CONSOLE_URL } from './nav-config'

export type NextPanel = 'admin' | 'console'

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

export function useNextPanel(): NextPanel {
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()
  return pathname.startsWith(NEXT_CONSOLE_URL(realm_name)) ? 'console' : 'admin'
}

export function useSwitchNextPanel() {
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()

  return (target: NextPanel) => {
    const adminRoot = NEXT_URL(realm_name)
    const consoleRoot = NEXT_CONSOLE_URL(realm_name)
    const current: NextPanel = pathname.startsWith(consoleRoot) ? 'console' : 'admin'
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
