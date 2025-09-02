import { Schemas } from '@/api/api.client'

export const WEBHOOK_CATEGORIES: Record<string, Schemas.WebhookTrigger[]> = {
  Client: [
    'client.created',
    'client.deleted',
    'client.updated',
    'client.role.created',
    'client.role.updated',
    'redirect_uri.created',
    'redirect_uri.deleted',
    'redirect_uri.updated',
  ],
  Realm: ['realm.created', 'realm.deleted', 'realm.settings.updated', 'realm.updated'],
  Role: ['role.created', 'role.updated'],
  User: [
    'user.assign.role',
    'user.bulk_deleted',
    'user.created',
    'user.credentials.deleted',
    'user.deleted',
    'user.unassign.role',
    'user.updated',
    'auth.reset_password',
  ],
  Webhook: ['webhook.created', 'webhook.deleted', 'webhook.updated'],
}

export const WEBHOOK_TRIGGER_LABELS: Record<Schemas.WebhookTrigger, string> = {
  'client.created': 'Client Created',
  'client.deleted': 'Client Deleted',
  'client.updated': 'Client Updated',
  'client.role.created': 'Client Role Created',
  'client.role.updated': 'Client Role Updated',
  'redirect_uri.created': 'Redirect URI Created',
  'redirect_uri.deleted': 'Redirect URI Deleted',
  'redirect_uri.updated': 'Redirect URI Updated',

  'realm.created': 'Realm Created',
  'realm.deleted': 'Realm Deleted',
  'realm.settings.updated': 'Realm Settings Updated',
  'realm.updated': 'Realm Updated',

  'role.created': 'Role Created',
  'role.updated': 'Role Updated',

  'user.assign.role': 'User Assigned Role',
  'user.bulk_deleted': 'User Bulk Deleted',
  'user.created': 'User Created',
  'user.credentials.deleted': 'User Deleted Credentials',
  'user.deleted': 'User Deleted',
  'user.unassign.role': 'User Unassigned Role',
  'user.updated': 'User Updated',
  'auth.reset_password': 'Auth Reset Password',

  'webhook.created': 'Webhook Created',
  'webhook.deleted': 'Webhook Deleted',
  'webhook.updated': 'Webhook Updated',
}

export const WEBHOOK_TRIGGER_DESCRIPTIONS: Record<Schemas.WebhookTrigger, string> = {
  'client.created': 'A new client has been created.',
  'client.deleted': 'A client has been deleted.',
  'client.updated': 'A client has been updated.',
  'client.role.created': 'A new client role has been created.',
  'client.role.updated': 'A client role has been updated.',
  'redirect_uri.created': 'A new redirect URI has been created.',
  'redirect_uri.deleted': 'A redirect URI has been deleted.',
  'redirect_uri.updated': 'A redirect URI has been updated.',

  'realm.created': 'A new realm has been created.',
  'realm.deleted': 'A realm has been deleted.',
  'realm.settings.updated': 'Realm settings have been updated.',
  'realm.updated': 'A realm has been updated.',

  'role.created': 'A new role has been created.',
  'role.updated': 'A role has been updated.',

  'user.assign.role': 'A user has been assigned a role.',
  'user.bulk_deleted': 'Multiple users have been deleted.',
  'user.created': 'A new user has been created.',
  'user.credentials.deleted': 'A user credentials have been deleted.',
  'user.deleted': 'A user has been deleted.',
  'user.unassign.role': 'A user has been unassigned a role.',
  'user.updated': 'A user has been updated.',
  'auth.reset_password': 'A user password has been reset.',

  'webhook.created': 'A new webhook has been created.',
  'webhook.deleted': 'A webhook has been deleted.',
  'webhook.updated': 'A webhook has been updated.',
}

export type WebhookCategory = {
  category: string
  events: {
    key: Schemas.WebhookTrigger
    label: string
    description: string
  }[]
}

export const getWebhookCategoriesForUI = (): WebhookCategory[] => {
  return Object.entries(WEBHOOK_CATEGORIES).map(([category, triggers]) => ({
    category,
    events: triggers.map((trigger) => ({
      key: trigger,
      label: WEBHOOK_TRIGGER_LABELS[trigger],
      description: WEBHOOK_TRIGGER_DESCRIPTIONS[trigger],
    })),
  }))
}

export const getAllWebhookTriggers = (): Schemas.WebhookTrigger[] => {
  return Object.values(WEBHOOK_CATEGORIES).flat()
}
