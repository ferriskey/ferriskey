import { Clock, FileKey, Globe, KeyRound, Power, PowerOff, ShieldCheck, Wrench } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { Choice } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { ClientProtocol } from '@/lib/client-protocol'

import MaintenanceSessionStrategy = Schemas.MaintenanceSessionStrategy

export type { ClientProtocol } from '@/lib/client-protocol'
export type ClientAuthentication = 'confidential' | 'public'
export type ClientState = 'enabled' | 'disabled'
export type MaintenanceState = 'open' | 'maintenance'

export type ClientTranslate = TFunction<'client'>

export const DEFAULT_PROTOCOL: ClientProtocol = 'openid-connect'

export const ASSIGNABLE_SCOPE_TYPES = ['default', 'optional'] as const

export const isClientProtocol = (value: string | null): value is ClientProtocol =>
  value === 'openid-connect' || value === 'saml'

export const clientStateOf = (enabled: boolean): ClientState =>
  enabled ? 'enabled' : 'disabled'

export const isEnabledState = (state: ClientState) => state === 'enabled'

export const clientAuthenticationOf = (publicClient: boolean): ClientAuthentication =>
  publicClient ? 'public' : 'confidential'

export const maintenanceStateOf = (enabled: boolean): MaintenanceState =>
  enabled ? 'maintenance' : 'open'

export const isMaintenanceState = (state: MaintenanceState) => state === 'maintenance'

export const protocolChoices = (t: ClientTranslate): Choice<ClientProtocol>[] => [
  {
    value: 'openid-connect',
    label: t('choices.protocol.openid_connect.label'),
    description: t('choices.protocol.openid_connect.description'),
    icon: KeyRound,
  },
  {
    value: 'saml',
    label: t('choices.protocol.saml.label'),
    description: t('choices.protocol.saml.description'),
    icon: FileKey,
  },
]

export const stateChoices = (t: ClientTranslate): Choice<ClientState>[] => [
  {
    value: 'enabled',
    label: t('choices.state.enabled.label'),
    description: t('choices.state.enabled.description'),
    icon: Power,
  },
  {
    value: 'disabled',
    label: t('choices.state.disabled.label'),
    description: t('choices.state.disabled.description'),
    icon: PowerOff,
  },
]

export const authenticationChoices = (t: ClientTranslate): Choice<ClientAuthentication>[] => [
  {
    value: 'confidential',
    label: t('choices.authentication.confidential.label'),
    description: t('choices.authentication.confidential.description'),
    icon: KeyRound,
  },
  {
    value: 'public',
    label: t('choices.authentication.public.label'),
    description: t('choices.authentication.public.description'),
    icon: Globe,
  },
]

export const lockedAuthenticationChoices = (
  t: ClientTranslate
): Choice<ClientAuthentication>[] =>
  authenticationChoices(t).map((choice) => ({
    ...choice,
    disabledReason: t('choices.authentication.immutable'),
  }))

export const maintenanceStateChoices = (t: ClientTranslate): Choice<MaintenanceState>[] => [
  {
    value: 'open',
    label: t('choices.maintenance_state.open.label'),
    description: t('choices.maintenance_state.open.description'),
    icon: ShieldCheck,
  },
  {
    value: 'maintenance',
    label: t('choices.maintenance_state.maintenance.label'),
    description: t('choices.maintenance_state.maintenance.description'),
    icon: Wrench,
  },
]

export const sessionStrategyChoices = (
  t: ClientTranslate
): Choice<MaintenanceSessionStrategy>[] => [
  {
    value: 'expire',
    label: t('choices.session_strategy.expire.label'),
    description: t('choices.session_strategy.expire.description'),
    icon: Clock,
  },
  {
    value: 'terminate',
    label: t('choices.session_strategy.terminate.label'),
    description: t('choices.session_strategy.terminate.description'),
    icon: PowerOff,
  },
]
