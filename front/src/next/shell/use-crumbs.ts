import { useLocation } from 'react-router-dom'
import { useCrumbStore } from './crumb-store'

export interface Crumb {
  label: string
  to?: string
}

const labels: Record<string, string> = {
  overview: 'Overview',
  clients: 'Clients',
  users: 'Users',
  roles: 'Roles',
  'client-scopes': 'Client Scopes',
  organizations: 'Organizations',
  'identity-providers': 'Identity Providers',
  'user-federation': 'User Federation',
  'realm-settings': 'Realm Settings',
  'email-templates': 'Emails',
  webhooks: 'Webhooks',
  portal: 'Portal',
  seawatch: 'Sea Watch',
  compass: 'Compass',
  settings: 'Settings',
  permissions: 'Permissions',
  credentials: 'Credentials',
  mappers: 'Protocol Mappers',
  scopes: 'Scopes',
  'role-mapping': 'Role mapping',
  attributes: 'Attributes',
  sessions: 'Sessions',
  account: 'Account',
  theme: 'Theme',
  sync: 'Sync',
  themes: 'Themes',
  layouts: 'Layouts',
  pages: 'Pages',
  events: 'Events',
  deliveries: 'Deliveries',
  tokens: 'Tokens',
  login: 'Login',
  general: 'General',
  'password-policy': 'Password Policy',
  create: 'New',
}

export function useCrumbs(): Crumb[] {
  const { pathname } = useLocation()
  const named = useCrumbStore((state) => state.labels)
  const parts = pathname.split('/').filter(Boolean)

  const rest = parts.slice(3)
  if (rest.length === 0) return [{ label: 'Overview' }]

  const crumbs: Crumb[] = []
  let acc = `/${parts.slice(0, 3).join('/')}`

  rest.forEach((part) => {
    acc += `/${part}`
    crumbs.push({ label: named[part] ?? labels[part] ?? part, to: acc })
  })

  return crumbs
}
