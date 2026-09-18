import {
  Building2,
  Compass,
  Database,
  Eye,
  Fingerprint,
  Gauge,
  KeyRound,
  LayoutGrid,
  Link2,
  Mail,
  Palette,
  Settings,
  Shield,
  Users,
  Webhook,
  type LucideIcon,
} from 'lucide-react'

export interface NavItem {
  to: string
  icon: LucideIcon
  disabled?: boolean
}

export interface NavSection {
  key: string
  items: NavItem[]
}

export const navSections: NavSection[] = [
  {
    key: 'core',
    items: [
      { to: 'overview', icon: Gauge },
      { to: 'clients', icon: LayoutGrid },
      { to: 'users', icon: Users },
      { to: 'roles', icon: Shield },
      { to: 'client-scopes', icon: KeyRound },
      { to: 'organizations', icon: Building2 },
    ],
  },
  {
    key: 'configuration',
    items: [
      { to: 'realm-settings', icon: Settings },
      { to: 'portal', icon: Palette },
      { to: 'email-templates', icon: Mail },
      { to: 'webhooks', icon: Webhook },
      { to: 'identity-providers', icon: Link2 },
      { to: 'user-federation', icon: Database },
      { to: 'authentication', icon: Fingerprint, disabled: true },
    ],
  },
  {
    key: 'security',
    items: [
      { to: 'seawatch', icon: Eye },
      { to: 'compass', icon: Compass },
    ],
  },
]

export const navSegmentKey = (segment: string) => `nav.segment.${segment}`

export const navSectionKey = (key: string) => `nav.section.${key}`
