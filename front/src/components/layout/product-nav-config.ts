import {
  Activity,
  Fingerprint,
  Globe,
  Inbox,
  KeyRound,
  LayoutGrid,
  Logs,
  type LucideIcon,
  Mail,
  MonitorSmartphone,
  Palette,
  Shield,
  ShieldUser,
  Users,
} from 'lucide-react'

export type ProductSectionKey = 'activity' | 'users' | 'auth' | 'branding'

export interface ProductSubItem {
  label: string
  description?: string
  icon: LucideIcon
  to: (realmName: string) => string
  match: (pathname: string, realmName: string) => boolean
}

export interface ProductSection {
  key: ProductSectionKey
  label: string
  icon: LucideIcon
  to: (realmName: string) => string
  match: (pathname: string, realmName: string) => boolean
  subItems: ProductSubItem[]
}

const realmPath = (realmName: string) => `/realms/${realmName}`
const startsWith = (path: string, prefix: string) => path === prefix || path.startsWith(`${prefix}/`)

export const productSections: ProductSection[] = [
  {
    key: 'activity',
    label: 'Activity',
    icon: LayoutGrid,
    to: (r) => `${realmPath(r)}/activity/live`,
    match: (p, r) =>
      startsWith(p, `${realmPath(r)}/activity`) ||
      startsWith(p, `${realmPath(r)}/overview`) ||
      startsWith(p, `${realmPath(r)}/compass`) ||
      startsWith(p, `${realmPath(r)}/seawatch`),
    subItems: [
      {
        label: 'Live',
        description: 'Sign-ups & logins live',
        icon: Activity,
        to: (r) => `${realmPath(r)}/activity/live`,
        match: (p, r) =>
          startsWith(p, `${realmPath(r)}/activity/live`) ||
          startsWith(p, `${realmPath(r)}/overview`),
      },
      {
        label: 'Logs & events',
        description: 'Searchable event feed',
        icon: Logs,
        to: (r) => `${realmPath(r)}/activity/logs`,
        match: (p, r) =>
          startsWith(p, `${realmPath(r)}/activity/logs`) ||
          startsWith(p, `${realmPath(r)}/seawatch`) ||
          startsWith(p, `${realmPath(r)}/compass`),
      },
      {
        label: 'Sessions',
        description: 'Active devices',
        icon: MonitorSmartphone,
        to: (r) => `${realmPath(r)}/activity/sessions`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/activity/sessions`),
      },
      {
        label: 'Message delivery',
        description: 'Emails & webhooks',
        icon: Inbox,
        to: (r) => `${realmPath(r)}/activity/messages`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/activity/messages`),
      },
    ],
  },
  {
    key: 'users',
    label: 'User management',
    icon: Users,
    to: (r) => `${realmPath(r)}/user-management/identities`,
    match: (p, r) =>
      startsWith(p, `${realmPath(r)}/user-management`) ||
      startsWith(p, `${realmPath(r)}/users`) ||
      startsWith(p, `${realmPath(r)}/organizations`) ||
      startsWith(p, `${realmPath(r)}/roles`),
    subItems: [
      {
        label: 'Identities',
        description: 'Customer accounts',
        icon: Users,
        to: (r) => `${realmPath(r)}/user-management/identities`,
        match: (p, r) =>
          startsWith(p, `${realmPath(r)}/user-management/identities`) ||
          startsWith(p, `${realmPath(r)}/users`),
      },
      {
        label: 'Organizations',
        description: 'B2B grouping',
        icon: Globe,
        to: (r) => `${realmPath(r)}/organizations`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/organizations`),
      },
      {
        label: 'Roles',
        description: 'Permissions & policies',
        icon: ShieldUser,
        to: (r) => `${realmPath(r)}/roles/overview`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/roles`),
      },
    ],
  },
  {
    key: 'auth',
    label: 'Authentication',
    icon: Shield,
    to: (r) => `${realmPath(r)}/realm-settings`,
    match: (p, r) =>
      startsWith(p, `${realmPath(r)}/realm-settings`) ||
      startsWith(p, `${realmPath(r)}/identity-providers`) ||
      startsWith(p, `${realmPath(r)}/user-federation`),
    subItems: [
      {
        label: 'Sign-in methods',
        description: 'Passkey, magic link, MFA',
        icon: Fingerprint,
        to: (r) => `${realmPath(r)}/realm-settings`,
        match: (p, r) => p === `${realmPath(r)}/realm-settings` || startsWith(p, `${realmPath(r)}/realm-settings/login`),
      },
      {
        label: 'Identity providers',
        description: 'Social & SSO',
        icon: Globe,
        to: (r) => `${realmPath(r)}/identity-providers`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/identity-providers`),
      },
      {
        label: 'User federation',
        description: 'External directories',
        icon: Users,
        to: (r) => `${realmPath(r)}/user-federation`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/user-federation`),
      },
      {
        label: 'Password policy',
        description: 'Strength requirements',
        icon: KeyRound,
        to: (r) => `${realmPath(r)}/realm-settings/password-policy`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/realm-settings/password-policy`),
      },
    ],
  },
  {
    key: 'branding',
    label: 'Branding',
    icon: Palette,
    to: (r) => `${realmPath(r)}/email-templates`,
    match: (p, r) => startsWith(p, `${realmPath(r)}/email-templates`),
    subItems: [
      {
        label: 'Email templates',
        description: 'Transactional emails',
        icon: Mail,
        to: (r) => `${realmPath(r)}/email-templates`,
        match: (p, r) => startsWith(p, `${realmPath(r)}/email-templates`),
      },
    ],
  },
]

export function findActiveSection(pathname: string, realmName: string): ProductSection | undefined {
  return productSections.find((s) => s.match(pathname, realmName))
}
