import { useTranslation } from 'react-i18next'
import { useLocation } from 'react-router-dom'
import { useCrumbStore } from '../crumb-store'
import { navSegmentKey } from './nav'

export interface Crumb {
  label: string
  to?: string
}

const NAMED_SEGMENTS = [
  'overview',
  'clients',
  'users',
  'roles',
  'client-scopes',
  'organizations',
  'identity-providers',
  'user-federation',
  'realm-settings',
  'email-templates',
  'webhooks',
  'portal',
  'seawatch',
  'compass',
  'settings',
  'permissions',
  'credentials',
  'mappers',
  'scopes',
  'role-mapping',
  'attributes',
  'sessions',
  'account',
  'theme',
  'sync',
  'themes',
  'layouts',
  'pages',
  'events',
  'deliveries',
  'tokens',
  'login',
  'general',
  'password-policy',
  'create',
]

const ROOT_SEGMENT = 'overview'

export function useCrumbs(): Crumb[] {
  const { t } = useTranslation()
  const { pathname } = useLocation()
  const named = useCrumbStore((state) => state.labels)
  const parts = pathname.split('/').filter(Boolean)

  const label = (segment: string) =>
    NAMED_SEGMENTS.includes(segment) ? t(navSegmentKey(segment)) : segment

  const rest = parts.slice(2)
  if (rest.length === 0) return [{ label: label(ROOT_SEGMENT) }]

  const crumbs: Crumb[] = []
  let acc = `/${parts.slice(0, 2).join('/')}`

  rest.forEach((part) => {
    acc += `/${part}`
    crumbs.push({ label: named[part] ?? label(part), to: acc })
  })

  return crumbs
}
