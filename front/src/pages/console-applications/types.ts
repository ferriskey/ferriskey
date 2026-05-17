import { Schemas } from '@/api/api.client'
import { ApplicationType } from '@/routes/sub-router/applications.router'
import { Cpu, Globe, Server, Smartphone, type LucideIcon } from 'lucide-react'

export interface ApplicationTypeMeta {
  key: ApplicationType
  label: string
  short: string
  description: string
  icon: LucideIcon
  tone: 'blue' | 'emerald' | 'amber' | 'violet'
  /** Hint about the auth flow this type uses. */
  flow: string
}

export const APPLICATION_TYPES: ApplicationTypeMeta[] = [
  {
    key: 'native',
    label: 'Native',
    short: 'Native',
    description: 'Mobile or desktop app installed on the user\u2019s device.',
    icon: Smartphone,
    tone: 'blue',
    flow: 'Authorization Code + PKCE',
  },
  {
    key: 'spa',
    label: 'Single-page app',
    short: 'SPA',
    description: 'JavaScript app running entirely in the browser (React, Vue, Svelte\u2026).',
    icon: Globe,
    tone: 'emerald',
    flow: 'Authorization Code + PKCE',
  },
  {
    key: 'web',
    label: 'Regular web app',
    short: 'Web',
    description: 'Server-rendered application that can keep a client secret safe.',
    icon: Server,
    tone: 'amber',
    flow: 'Authorization Code',
  },
  {
    key: 'm2m',
    label: 'Machine to Machine',
    short: 'M2M',
    description: 'Backend service or daemon authenticating without a user.',
    icon: Cpu,
    tone: 'violet',
    flow: 'Client Credentials',
  },
]

export const APPLICATION_TONE: Record<
  ApplicationTypeMeta['tone'],
  { bg: string; fg: string; border: string }
> = {
  blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500', border: 'border-blue-500/30' },
  emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500', border: 'border-emerald-500/30' },
  amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500', border: 'border-amber-500/30' },
  violet: { bg: 'bg-violet-500/10', fg: 'text-violet-500', border: 'border-violet-500/30' },
}

/** Maps a FerrisKey client to its CIAM application-type bucket. */
export function inferApplicationType(client: Schemas.Client): ApplicationType {
  if (client.service_account_enabled) return 'm2m'
  if (client.client_type === 'public') return client.public_client ? 'spa' : 'native'
  return 'web'
}

export function getApplicationTypeMeta(type: ApplicationType): ApplicationTypeMeta {
  return APPLICATION_TYPES.find((t) => t.key === type) ?? APPLICATION_TYPES[0]
}
