export { formatRelative, formatTimestamp } from '@/utils/format-date'

import { Schemas } from '@/api/api.client'

import SecurityEvent = Schemas.SecurityEvent
import SecurityEventType = Schemas.SecurityEventType

export const eventLabels: Record<SecurityEventType, string> = {
  login_success: 'Login succeeded',
  login_failure: 'Login failed',
  password_reset: 'Password reset',
  password_reset_requested: 'Password reset requested',
  password_reset_completed: 'Password reset completed',
  user_created: 'User created',
  user_email_verified: 'Email address verified',
  user_deleted: 'User deleted',
  role_assigned: 'Role assigned',
  role_unassigned: 'Role unassigned',
  role_created: 'Role created',
  role_removed: 'Role removed',
  client_created: 'Client created',
  client_deleted: 'Client deleted',
  client_secret_rotated: 'Client secret rotated',
  client_secret_viewed: 'Client secret viewed',
  realm_config_changed: 'Realm configuration changed',
  email_not_sent: 'Email delivery failed',
  email_sent: 'Email delivered',
  client_maintenance_enabled: 'Client maintenance enabled',
  client_maintenance_disabled: 'Client maintenance disabled',
  session_created: 'Session opened',
  session_revoked: 'Session revoked',
  identity_provider_link_removed: 'Identity provider link removed',
  unknown: 'Unrecognised event',
}

export const eventLabel = (event: SecurityEvent) =>
  eventLabels[event.event_type] ?? event.event_type

const authenticationEvents: SecurityEventType[] = [
  'login_success',
  'login_failure',
  'session_created',
  'session_revoked',
  'identity_provider_link_removed',
]

const credentialEvents: SecurityEventType[] = [
  'password_reset',
  'password_reset_requested',
  'password_reset_completed',
  'user_email_verified',
]

const administrationEvents: SecurityEventType[] = [
  'user_created',
  'user_deleted',
  'role_assigned',
  'role_unassigned',
  'role_created',
  'role_removed',
  'client_created',
  'client_deleted',
  'client_secret_rotated',
  'client_secret_viewed',
  'realm_config_changed',
  'client_maintenance_enabled',
  'client_maintenance_disabled',
]

export const isAuthenticationEvent = (event: SecurityEvent) =>
  authenticationEvents.includes(event.event_type)

export const isCredentialEvent = (event: SecurityEvent) =>
  credentialEvents.includes(event.event_type)

export const isAdministrationEvent = (event: SecurityEvent) =>
  administrationEvents.includes(event.event_type)

const asRecord = (details: unknown): Record<string, unknown> | null =>
  typeof details === 'object' && details !== null && !Array.isArray(details)
    ? (details as Record<string, unknown>)
    : null

export const eventReason = (event: SecurityEvent) => {
  const details = asRecord(event.details)
  if (!details) return null
  const reason = details.reason
  const code = details.error_code
  return {
    reason: typeof reason === 'string' ? reason : null,
    errorCode: typeof code === 'string' ? code : null,
  }
}

export const eventDetailSummary = (event: SecurityEvent) => {
  const details = asRecord(event.details)
  if (!details) return null
  const entries = Object.entries(details).filter(
    ([key]) => key !== 'reason' && key !== 'error_code'
  )
  if (entries.length === 0) return null
  return entries.map(([key, value]) => `${key}=${String(value)}`).join(' · ')
}

export const actorLabel = (event: SecurityEvent) =>
  event.actor_id ?? event.target_id ?? null

export const eventFamilies: Record<string, readonly SecurityEventType[]> = {
  authentication: authenticationEvents,
  credentials: credentialEvents,
  administration: administrationEvents,
}
