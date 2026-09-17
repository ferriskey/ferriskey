import { Cpu, Globe, MonitorSmartphone, Server, Smartphone } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { Choice, PillTone } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client
import CreateClientValidator = Schemas.CreateClientValidator
import { DEFAULT_CLIENT_PROTOCOL } from '@/lib/client-protocol'

export type ConsoleTranslate = TFunction<'console'>

export type ApplicationType = 'native' | 'spa' | 'web' | 'm2m' | 'device'

export const APPLICATION_TYPES: readonly ApplicationType[] = [
  'native',
  'spa',
  'web',
  'm2m',
  'device',
]

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

interface ApplicationTypeShape {
  tone: PillTone
  usesAuthorizationCode: boolean
  holdsSecret: boolean
}

const APPLICATION_TYPE_SHAPES: Record<ApplicationType, ApplicationTypeShape> = {
  native: { tone: 'info', usesAuthorizationCode: true, holdsSecret: true },
  spa: { tone: 'success', usesAuthorizationCode: true, holdsSecret: false },
  web: { tone: 'primary', usesAuthorizationCode: true, holdsSecret: true },
  m2m: { tone: 'violet', usesAuthorizationCode: false, holdsSecret: true },
  device: { tone: 'neutral', usesAuthorizationCode: false, holdsSecret: false },
}

const APPLICATION_TYPE_ICONS = {
  native: Smartphone,
  spa: Globe,
  web: Server,
  m2m: Cpu,
  device: MonitorSmartphone,
} as const

export const applicationTypeIcon = (type: ApplicationType) => APPLICATION_TYPE_ICONS[type]

export const isApplicationType = (value: string | null): value is ApplicationType =>
  APPLICATION_TYPES.some((type) => type === value)

export const applicationTypeMeta = (
  type: ApplicationType,
  t: ConsoleTranslate
): ApplicationTypeMeta => {
  const key = isApplicationType(type) ? type : APPLICATION_TYPES[0]

  return {
    key,
    label: t(`applications.types.${key}.label`),
    short: t(`applications.types.${key}.short`),
    description: t(`applications.types.${key}.description`),
    flow: t(`applications.types.${key}.flow`),
    ...APPLICATION_TYPE_SHAPES[key],
  }
}

export const applicationTypeMetas = (t: ConsoleTranslate): ApplicationTypeMeta[] =>
  APPLICATION_TYPES.map((type) => applicationTypeMeta(type, t))

export const applicationTypeChoices = (t: ConsoleTranslate): Choice<ApplicationType>[] =>
  applicationTypeMetas(t).map((meta) => ({
    value: meta.key,
    label: meta.label,
    description: t('applications.types.choice_description', {
      description: meta.description,
      flow: meta.flow,
    }),
    icon: APPLICATION_TYPE_ICONS[meta.key],
  }))

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

interface ApplicationFieldsShape {
  showCallbacks: boolean
  callbackRequired: boolean
  showOrigins: boolean
}

const APPLICATION_FIELD_SHAPES: Record<ApplicationType, ApplicationFieldsShape> = {
  native: { showCallbacks: true, callbackRequired: true, showOrigins: false },
  spa: { showCallbacks: true, callbackRequired: true, showOrigins: true },
  web: { showCallbacks: true, callbackRequired: true, showOrigins: false },
  m2m: { showCallbacks: false, callbackRequired: false, showOrigins: false },
  device: { showCallbacks: false, callbackRequired: false, showOrigins: false },
}

export const applicationFields = (
  type: ApplicationType,
  t: ConsoleTranslate
): ApplicationFieldsConfig => {
  const shape = APPLICATION_FIELD_SHAPES[type]

  return {
    ...shape,
    callbacksLabel: shape.showCallbacks ? t('applications.fields.callbacks.label') : '',
    callbacksHint: shape.showCallbacks ? t(`applications.fields.callbacks.hint.${type}`) : '',
    callbacksPlaceholder: shape.showCallbacks
      ? t(`applications.fields.callbacks.placeholder.${type}`)
      : '',
    originsLabel: shape.showOrigins ? t('applications.fields.origins.label') : '',
    originsHint: shape.showOrigins ? t('applications.fields.origins.hint') : '',
    originsPlaceholder: shape.showOrigins ? t('applications.fields.origins.placeholder') : '',
  }
}

export const applicationFieldsShape = (type: ApplicationType): ApplicationFieldsShape =>
  APPLICATION_FIELD_SHAPES[type]
