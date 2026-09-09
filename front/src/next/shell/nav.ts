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
  label: string
  to: string
  icon: LucideIcon
  ready?: boolean
  disabled?: boolean
}

export interface NavSection {
  title: string
  items: NavItem[]
}

export const navSections: NavSection[] = [
  {
    title: 'Core',
    items: [
      { label: 'Overview', to: 'overview', icon: Gauge },
      { label: 'Clients', to: 'clients', icon: LayoutGrid },
      { label: 'Users', to: 'users', icon: Users },
      { label: 'Roles', to: 'roles', icon: Shield, ready: true },
      { label: 'Client Scopes', to: 'client-scopes', icon: KeyRound },
      { label: 'Organizations', to: 'organizations', icon: Building2 },
    ],
  },
  {
    title: 'Configuration',
    items: [
      { label: 'Realm Settings', to: 'realm-settings', icon: Settings },
      { label: 'Portal', to: 'portal', icon: Palette },
      { label: 'Emails', to: 'email-templates', icon: Mail },
      { label: 'Webhooks', to: 'webhooks', icon: Webhook },
      { label: 'Identity Providers', to: 'identity-providers', icon: Link2 },
      { label: 'User Federation', to: 'user-federation', icon: Database },
      { label: 'Authentication', to: 'authentication', icon: Fingerprint, disabled: true },
    ],
  },
  {
    title: 'Security',
    items: [
      { label: 'Sea Watch', to: 'seawatch', icon: Eye },
      { label: 'Compass', to: 'compass', icon: Compass },
    ],
  },
]
