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
  icon: LucideIcon
  segment: string
}

export interface ConsoleSection {
  key: ConsoleSectionKey
  icon: LucideIcon
  segment: string
  subItems: ConsoleSubItem[]
}

export const consoleSectionKey = (key: string) => `console_nav.section.${key}`

export const consoleItemKey = (key: string, field: 'label' | 'description') =>
  `console_nav.item.${key}.${field}`


export const consoleSections: ConsoleSection[] = [
  {
    key: 'activity',
    icon: LayoutGrid,
    segment: 'activity',
    subItems: [
      {
        key: 'live',
        icon: Activity,
        segment: 'activity/live',
      },
      {
        key: 'logs',
        icon: Logs,
        segment: 'activity/logs',
      },
      {
        key: 'sessions',
        icon: MonitorSmartphone,
        segment: 'activity/sessions',
      },
      {
        key: 'messages',
        icon: Inbox,
        segment: 'activity/messages',
      },
    ],
  },
  {
    key: 'users',
    icon: Users,
    segment: 'user-management',
    subItems: [
      {
        key: 'identities',
        icon: Users,
        segment: 'user-management/identities',
      },
      {
        key: 'organizations',
        icon: Globe,
        segment: 'user-management/organizations',
      },
      {
        key: 'roles',
        icon: ShieldUser,
        segment: 'user-management/roles',
      },
    ],
  },
  {
    key: 'applications',
    icon: Boxes,
    segment: 'applications',
    subItems: [
      {
        key: 'all',
        icon: Boxes,
        segment: 'applications',
      },
    ],
  },
  {
    key: 'auth',
    icon: Shield,
    segment: 'authentication',
    subItems: [
      {
        key: 'sign-in-methods',
        icon: Fingerprint,
        segment: 'authentication/sign-in-methods',
      },
      {
        key: 'identity-providers',
        icon: Globe,
        segment: 'authentication/identity-providers',
      },
      {
        key: 'password-policy',
        icon: KeyRound,
        segment: 'authentication/password-policy',
      },
    ],
  },
  {
    key: 'branding',
    icon: Palette,
    segment: 'branding',
    subItems: [
      {
        key: 'email-templates',
        icon: Mail,
        segment: 'branding/email-templates',
      },
      {
        key: 'themes',
        icon: Palette,
        segment: 'branding/themes',
      },
      {
        key: 'layouts',
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
