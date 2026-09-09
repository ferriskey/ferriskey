import { Cpu, Globe, MonitorSmartphone, Server, Smartphone } from 'lucide-react'
import type { Choice, PillTone } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client
import CreateClientValidator = Schemas.CreateClientValidator
import { DEFAULT_CLIENT_PROTOCOL } from '@/lib/client-protocol'

export type ApplicationType = 'native' | 'spa' | 'web' | 'm2m' | 'device'

export interface ApplicationTypeMeta {
  key: ApplicationType
  label: string
  short: string
  description: string
  flow: string
  tone: PillTone
  usesAuthorizationCode: boolean
  holdsSecret: boolean
}

export const APPLICATION_TYPE_METAS: ApplicationTypeMeta[] = [
  {
    key: 'native',
    label: 'Mobile or desktop',
    short: 'Native',
    description: 'Installed on the user’s device, so its code can be read by anyone holding it.',
    flow: 'Authorization Code + PKCE',
    tone: 'info',
    usesAuthorizationCode: true,
    holdsSecret: true,
  },
  {
    key: 'spa',
    label: 'Single-page app',
    short: 'SPA',
    description: 'Runs entirely in the browser, so it gets no secret and relies on PKCE.',
    flow: 'Authorization Code + PKCE',
    tone: 'success',
    usesAuthorizationCode: true,
    holdsSecret: false,
  },
  {
    key: 'web',
    label: 'Server-rendered web app',
    short: 'Web',
    description: 'Keeps a secret on your server and exchanges the code there.',
    flow: 'Authorization Code',
    tone: 'primary',
    usesAuthorizationCode: true,
    holdsSecret: true,
  },
  {
    key: 'm2m',
    label: 'Machine to machine',
    short: 'M2M',
    description: 'A backend or daemon signing in as itself, with no user in front of it.',
    flow: 'Client Credentials',
    tone: 'violet',
    usesAuthorizationCode: false,
    holdsSecret: true,
  },
  {
    key: 'device',
    label: 'Device or CLI',
    short: 'Device',
    description: 'No browser on the device: the user approves the request on another screen.',
    flow: 'Device Authorization Grant (RFC 8628)',
    tone: 'neutral',
    usesAuthorizationCode: false,
    holdsSecret: false,
  },
]

const APPLICATION_TYPE_ICONS = {
  native: Smartphone,
  spa: Globe,
  web: Server,
  m2m: Cpu,
  device: MonitorSmartphone,
} as const

export const applicationTypeIcon = (type: ApplicationType) => APPLICATION_TYPE_ICONS[type]

export const isApplicationType = (value: string | null): value is ApplicationType =>
  APPLICATION_TYPE_METAS.some((meta) => meta.key === value)

export const applicationTypeMeta = (type: ApplicationType): ApplicationTypeMeta =>
  APPLICATION_TYPE_METAS.find((meta) => meta.key === type) ?? APPLICATION_TYPE_METAS[0]

export const applicationTypeChoices: Choice<ApplicationType>[] = APPLICATION_TYPE_METAS.map(
  (meta) => ({
    value: meta.key,
    label: meta.label,
    description: `${meta.description} Signs in with ${meta.flow}.`,
    icon: APPLICATION_TYPE_ICONS[meta.key],
  })
)

export function inferApplicationType(client: Client): ApplicationType {
  if (client.service_account_enabled) return 'm2m'
  if (client.oauth_device_code_grant_enabled && (client.redirect_uris?.length ?? 0) === 0) {
    return 'device'
  }
  if (client.client_type === 'public') return client.public_client ? 'spa' : 'native'
  return 'web'
}

export function createPayloadFor(
  type: ApplicationType,
  name: string,
  clientId: string
): CreateClientValidator {
  const base = {
    name,
    client_id: clientId,
    enabled: true,
    protocol: DEFAULT_CLIENT_PROTOCOL,
    oauth_device_code_grant_enabled: false,
    direct_access_grants_enabled: false,
  }

  switch (type) {
    case 'native':
      return {
        ...base,
        client_type: 'public',
        public_client: false,
        service_account_enabled: false,
      }
    case 'spa':
      return {
        ...base,
        client_type: 'public',
        public_client: true,
        service_account_enabled: false,
      }
    case 'web':
      return {
        ...base,
        client_type: 'confidential',
        public_client: false,
        service_account_enabled: false,
      }
    case 'm2m':
      return {
        ...base,
        client_type: 'confidential',
        public_client: false,
        service_account_enabled: true,
      }
    case 'device':
      return {
        ...base,
        client_type: 'public',
        public_client: true,
        service_account_enabled: false,
        oauth_device_code_grant_enabled: true,
      }
  }
}

export interface ApplicationFieldsConfig {
  showCallbacks: boolean
  callbacksLabel: string
  callbacksHint: string
  callbacksPlaceholder: string
  callbackRequired: boolean
  showOrigins: boolean
  originsLabel: string
  originsHint: string
  originsPlaceholder: string
}

const NO_URLS = {
  showCallbacks: false,
  callbacksLabel: '',
  callbacksHint: '',
  callbacksPlaceholder: '',
  callbackRequired: false,
  showOrigins: false,
  originsLabel: '',
  originsHint: '',
  originsPlaceholder: '',
}

export const APPLICATION_FIELDS: Record<ApplicationType, ApplicationFieldsConfig> = {
  native: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint:
      'Where the sign-in sends the user back. Use a custom scheme such as com.acme.app://callback on mobile, or http://localhost for desktop development. A callback that is not listed is refused.',
    callbacksPlaceholder: 'com.acme.app://callback',
    callbackRequired: true,
    showOrigins: false,
    originsLabel: '',
    originsHint: '',
    originsPlaceholder: '',
  },
  spa: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint:
      'Where users land after sign-in. A callback that is not listed is refused, so add every environment you deploy to.',
    callbacksPlaceholder: 'https://app.acme.com/callback',
    callbackRequired: true,
    showOrigins: true,
    originsLabel: 'Allowed web origins',
    originsHint:
      'Origins allowed to call FerrisKey from the browser. Scheme, host and port only — no path. Enter + to derive them from the callback URLs above. Without a matching origin the browser blocks the call.',
    originsPlaceholder: 'https://app.acme.com',
  },
  web: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint:
      'The server-side endpoint that exchanges the authorization code for tokens. A callback that is not listed is refused.',
    callbacksPlaceholder: 'https://app.acme.com/auth/callback',
    callbackRequired: true,
    showOrigins: false,
    originsLabel: '',
    originsHint: '',
    originsPlaceholder: '',
  },
  m2m: { ...NO_URLS },
  device: { ...NO_URLS },
}
