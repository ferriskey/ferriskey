import { Schemas } from '@/api/api.client'
import { ApplicationType } from '@/routes/sub-router/applications.router'
import { Cpu, Globe, MonitorSmartphone, Server, Smartphone, type LucideIcon } from 'lucide-react'

export interface ApplicationTypeMeta {
  key: ApplicationType
  label: string
  short: string
  description: string
  icon: LucideIcon
  tone: 'blue' | 'emerald' | 'amber' | 'violet' | 'rose'
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
  {
    key: 'device',
    label: 'Device / CLI',
    short: 'Device',
    description:
      'Browserless client (CLI, IoT, smart TV) — the user approves access on a separate device.',
    icon: MonitorSmartphone,
    tone: 'rose',
    flow: 'Device Authorization Grant (RFC 8628)',
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
  rose: { bg: 'bg-rose-500/10', fg: 'text-rose-500', border: 'border-rose-500/30' },
}

/** Maps a FerrisKey client to its CIAM application-type bucket. */
export function inferApplicationType(client: Schemas.Client): ApplicationType {
  if (client.service_account_enabled) return 'm2m'
  // Device flow is opt-in via the per-client toggle. A public client with the
  // device grant on and no redirect URIs is, by construction, a device/CLI
  // client — surface it as such rather than mis-labelling it 'spa'/'native'.
  if (
    client.oauth_device_code_grant_enabled &&
    (client.redirect_uris?.length ?? 0) === 0
  ) {
    return 'device'
  }
  if (client.client_type === 'public') return client.public_client ? 'spa' : 'native'
  return 'web'
}

export function getApplicationTypeMeta(type: ApplicationType): ApplicationTypeMeta {
  return APPLICATION_TYPES.find((t) => t.key === type) ?? APPLICATION_TYPES[0]
}
