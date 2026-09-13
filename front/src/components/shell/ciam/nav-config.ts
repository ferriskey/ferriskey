import { CONSOLE_URL } from '@/routes/router'
import {
  Activity,
  Boxes,
  Fingerprint,
  Globe,
  Inbox,
  KeyRound,
  LayoutGrid,
  LayoutTemplate,
  Logs,
  Mail,
  MonitorSmartphone,
  Palette,
  Shield,
  ShieldUser,
  Users,
  type LucideIcon,
} from 'lucide-react'

export type ConsoleSectionKey =
  | 'activity'
  | 'users'
  | 'applications'
  | 'auth'
  | 'branding'

export interface ConsoleSubItem {
  key: string
  label: string
  description: string
  icon: LucideIcon
  segment: string
}

export interface ConsoleSection {
  key: ConsoleSectionKey
  label: string
  icon: LucideIcon
  segment: string
  subItems: ConsoleSubItem[]
}


export const consoleSections: ConsoleSection[] = [
  {
    key: 'activity',
    label: 'Activity',
    icon: LayoutGrid,
    segment: 'activity',
    subItems: [
      {
        key: 'live',
        label: 'Live',
        description: 'Sign-ups and logins as they happen',
        icon: Activity,
        segment: 'activity/live',
      },
      {
        key: 'logs',
        label: 'Logs & events',
        description: 'Searchable event feed',
        icon: Logs,
        segment: 'activity/logs',
      },
      {
        key: 'sessions',
        label: 'Sessions',
        description: 'Active devices',
        icon: MonitorSmartphone,
        segment: 'activity/sessions',
      },
      {
        key: 'messages',
        label: 'Message delivery',
        description: 'Emails and webhooks',
        icon: Inbox,
        segment: 'activity/messages',
      },
    ],
  },
  {
    key: 'users',
    label: 'User management',
    icon: Users,
    segment: 'user-management',
    subItems: [
      {
        key: 'identities',
        label: 'Identities',
        description: 'Customer accounts',
        icon: Users,
        segment: 'user-management/identities',
      },
      {
        key: 'organizations',
        label: 'Organizations',
        description: 'B2B grouping',
        icon: Globe,
        segment: 'user-management/organizations',
      },
      {
        key: 'roles',
        label: 'Roles',
        description: 'Permissions and policies',
        icon: ShieldUser,
        segment: 'user-management/roles',
      },
    ],
  },
  {
    key: 'applications',
    label: 'Applications',
    icon: Boxes,
    segment: 'applications',
    subItems: [
      {
        key: 'all',
        label: 'All applications',
        description: 'Mobile, SPA, web, M2M',
        icon: Boxes,
        segment: 'applications',
      },
    ],
  },
  {
    key: 'auth',
    label: 'Authentication',
    icon: Shield,
    segment: 'authentication',
    subItems: [
      {
        key: 'sign-in-methods',
        label: 'Sign-in methods',
        description: 'Passkey, magic link, MFA',
        icon: Fingerprint,
        segment: 'authentication/sign-in-methods',
      },
      {
        key: 'identity-providers',
        label: 'Identity providers',
        description: 'Social and SSO',
        icon: Globe,
        segment: 'authentication/identity-providers',
      },
      {
        key: 'password-policy',
        label: 'Password policy',
        description: 'Strength requirements',
        icon: KeyRound,
        segment: 'authentication/password-policy',
      },
    ],
  },
  {
    key: 'branding',
    label: 'Branding',
    icon: Palette,
    segment: 'branding',
    subItems: [
      {
        key: 'email-templates',
        label: 'Email templates',
        description: 'Transactional emails',
        icon: Mail,
        segment: 'branding/email-templates',
      },
      {
        key: 'themes',
        label: 'Themes',
        description: 'Portal appearance',
        icon: Palette,
        segment: 'branding/themes',
      },
      {
        key: 'layouts',
        label: 'Layouts',
        description: 'Portal page structures',
        icon: LayoutTemplate,
        segment: 'branding/layouts',
      },
    ],
  },
]

export const sectionUrl = (realm: string, section: ConsoleSection) =>
  `${CONSOLE_URL(realm)}/${section.subItems[0].segment}`

export const subItemUrl = (realm: string, item: ConsoleSubItem) =>
  `${CONSOLE_URL(realm)}/${item.segment}`

export function findActiveSection(pathname: string, realm: string) {
  const base = CONSOLE_URL(realm)
  return consoleSections.find(
    (s) => pathname === `${base}/${s.segment}` || pathname.startsWith(`${base}/${s.segment}/`)
  )
}

export function isSubItemActive(pathname: string, realm: string, item: ConsoleSubItem) {
  const url = subItemUrl(realm, item)
  return pathname === url || pathname.startsWith(`${url}/`)
}
