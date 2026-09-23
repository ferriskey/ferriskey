export { formatRelative, formatTimestamp } from '@/utils/format-date'

import { translate } from '@/lib/i18n'
import { Schemas } from '@/api/api.client'

import SecurityEvent = Schemas.SecurityEvent
import SecurityEventType = Schemas.SecurityEventType

export const catalogedEventTypes: Record<SecurityEventType, true> = {
  login_success: true,
  login_failure: true,
  password_reset: true,
  password_reset_requested: true,
  password_reset_completed: true,
  password_imported: true,
  user_created: true,
  user_email_verified: true,
  user_deleted: true,
  role_assigned: true,
  role_unassigned: true,
  role_created: true,
  role_removed: true,
  role_updated: true,
  role_permission_updated: true,
  client_created: true,
  client_deleted: true,
  client_secret_rotated: true,
  client_secret_viewed: true,
  realm_config_changed: true,
  email_not_sent: true,
  email_sent: true,
  client_maintenance_enabled: true,
  client_maintenance_disabled: true,
  session_created: true,
  session_revoked: true,
  webhook_delivery_exhausted: true,
  identity_provider_link_removed: true,
  unknown: true,
}

export const eventLabel = (event: SecurityEvent) =>
  catalogedEventTypes[event.event_type]
    ? translate(`seawatch:event.${event.event_type}`)
    : event.event_type

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
  'password_imported',
  'user_email_verified',
]

const administrationEvents: SecurityEventType[] = [
  'user_created',
  'user_deleted',
  'role_assigned',
  'role_unassigned',
  'role_created',
  'role_removed',
  'role_updated',
  'role_permission_updated',
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
