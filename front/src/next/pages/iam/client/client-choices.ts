import { Clock, FileKey, Globe, KeyRound, Power, PowerOff, ShieldCheck, Wrench } from 'lucide-react'
import type { Choice } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import MaintenanceSessionStrategy = Schemas.MaintenanceSessionStrategy

export type ClientProtocol = 'openid-connect' | 'saml'
export type ClientAuthentication = 'confidential' | 'public'
export type ClientState = 'enabled' | 'disabled'
export type MaintenanceState = 'open' | 'maintenance'

export const isClientProtocol = (value: string | null): value is ClientProtocol =>
  value === 'openid-connect' || value === 'saml'

export const protocolChoices: Choice<ClientProtocol>[] = [
  {
    value: 'openid-connect',
    label: 'OpenID Connect',
    description: 'Token-based sign-in, for web applications, SPAs, mobile apps and APIs.',
    icon: KeyRound,
  },
  {
    value: 'saml',
    label: 'SAML 2.0',
    description: 'Assertion-based sign-in, for applications that only speak SAML.',
    icon: FileKey,
  },
]

export const stateChoices: Choice<ClientState>[] = [
  {
    value: 'enabled',
    label: 'Enabled',
    description: 'The client can request an authentication.',
    icon: Power,
  },
  {
    value: 'disabled',
    label: 'Disabled',
    description: 'Every request is rejected, the configuration is kept.',
    icon: PowerOff,
  },
]

export const authenticationChoices: Choice<ClientAuthentication>[] = [
  {
    value: 'confidential',
    label: 'Confidential',
    description: 'The client proves its identity with a secret held server-side.',
    icon: KeyRound,
  },
  {
    value: 'public',
    label: 'Public',
    description: 'No secret: for SPAs, mobile applications and CLIs.',
    icon: Globe,
  },
]

const IMMUTABLE_AUTHENTICATION =
  'Set at creation: a client cannot move between confidential and public afterwards.'

export const lockedAuthenticationChoices: Choice<ClientAuthentication>[] =
  authenticationChoices.map((choice) => ({
    ...choice,
    disabledReason: IMMUTABLE_AUTHENTICATION,
  }))

export const maintenanceStateChoices: Choice<MaintenanceState>[] = [
  {
    value: 'open',
    label: 'Open',
    description: 'The client authenticates normally.',
    icon: ShieldCheck,
  },
  {
    value: 'maintenance',
    label: 'In maintenance',
    description: 'Only the whitelist gets through.',
    icon: Wrench,
  },
]

export const sessionStrategyChoices: Choice<MaintenanceSessionStrategy>[] = [
  {
    value: 'expire',
    label: 'Expire naturally',
    description: 'Open sessions live until they run out.',
    icon: Clock,
  },
  {
    value: 'terminate',
    label: 'Terminate immediately',
    description: 'Every session is cut on the spot.',
    icon: PowerOff,
  },
]
