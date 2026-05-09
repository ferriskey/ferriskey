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

const consolePath = (realmName: string) => `/realms/${realmName}/console`
const startsWith = (path: string, prefix: string) => path === prefix || path.startsWith(`${prefix}/`)

export const productSections: ProductSection[] = [
  {
    key: 'activity',
    label: 'Activity',
    icon: LayoutGrid,
    to: (r) => `${consolePath(r)}/activity/live`,
    match: (p, r) => startsWith(p, `${consolePath(r)}/activity`),
    subItems: [
      {
        label: 'Live',
        description: 'Sign-ups & logins live',
        icon: Activity,
        to: (r) => `${consolePath(r)}/activity/live`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/activity/live`),
      },
      {
        label: 'Logs & events',
        description: 'Searchable event feed',
        icon: Logs,
        to: (r) => `${consolePath(r)}/activity/logs`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/activity/logs`),
      },
      {
        label: 'Sessions',
        description: 'Active devices',
        icon: MonitorSmartphone,
        to: (r) => `${consolePath(r)}/activity/sessions`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/activity/sessions`),
      },
      {
        label: 'Message delivery',
        description: 'Emails & webhooks',
        icon: Inbox,
        to: (r) => `${consolePath(r)}/activity/messages`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/activity/messages`),
      },
    ],
  },
  {
    key: 'users',
    label: 'User management',
    icon: Users,
    to: (r) => `${consolePath(r)}/user-management/identities`,
    match: (p, r) => startsWith(p, `${consolePath(r)}/user-management`),
    subItems: [
      {
        label: 'Identities',
        description: 'Customer accounts',
        icon: Users,
        to: (r) => `${consolePath(r)}/user-management/identities`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/user-management/identities`),
      },
      {
        label: 'Organizations',
        description: 'B2B grouping',
        icon: Globe,
        to: (r) => `${consolePath(r)}/user-management/organizations`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/user-management/organizations`),
      },
      {
        label: 'Roles',
        description: 'Permissions & policies',
        icon: ShieldUser,
        to: (r) => `${consolePath(r)}/user-management/roles`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/user-management/roles`),
      },
    ],
  },
  {
    key: 'auth',
    label: 'Authentication',
    icon: Shield,
    to: (r) => `${consolePath(r)}/authentication`,
    match: (p, r) => startsWith(p, `${consolePath(r)}/authentication`),
    subItems: [
      {
        label: 'Sign-in methods',
        description: 'Passkey, magic link, MFA',
        icon: Fingerprint,
        to: (r) => `${consolePath(r)}/authentication/sign-in-methods`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/authentication/sign-in-methods`),
      },
      {
        label: 'Identity providers',
        description: 'Social & SSO',
        icon: Globe,
        to: (r) => `${consolePath(r)}/authentication/identity-providers`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/authentication/identity-providers`),
      },
      {
        label: 'User federation',
        description: 'External directories',
        icon: Users,
        to: (r) => `${consolePath(r)}/authentication/user-federation`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/authentication/user-federation`),
      },
      {
        label: 'Password policy',
        description: 'Strength requirements',
        icon: KeyRound,
        to: (r) => `${consolePath(r)}/authentication/password-policy`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/authentication/password-policy`),
      },
    ],
  },
  {
    key: 'branding',
    label: 'Branding',
    icon: Palette,
    to: (r) => `${consolePath(r)}/branding/email-templates`,
    match: (p, r) => startsWith(p, `${consolePath(r)}/branding`),
    subItems: [
      {
        label: 'Email templates',
        description: 'Transactional emails',
        icon: Mail,
        to: (r) => `${consolePath(r)}/branding/email-templates`,
        match: (p, r) => startsWith(p, `${consolePath(r)}/branding/email-templates`),
      },
    ],
  },
]

export function findActiveSection(pathname: string, realmName: string): ProductSection | undefined {
  return productSections.find((s) => s.match(pathname, realmName))
}
