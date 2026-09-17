import { Schemas } from '@/api/api.client'

export const WEBHOOK_CATEGORIES: Record<string, Schemas.WebhookTrigger[]> = {
  client: [
    'client.created',
    'client.deleted',
    'client.updated',
    'client.role.created',
    'client.role.updated',
    'redirect_uri.created',
    'redirect_uri.deleted',
    'redirect_uri.updated',
    'client.maintenance.enabled',
    'client.maintenance.disabled',
    'web_origin.created',
    'web_origin.deleted',
    'client.saml_config.updated',
    'client.saml_attribute_mapper.created',
    'client.saml_attribute_mapper.deleted',
  ],
  realm: ['realm.created', 'realm.deleted', 'realm.settings.updated', 'realm.updated'],
  role: ['role.created', 'role.updated'],
  user: [
    'user.role.assigned',
    'user.bulk_deleted',
    'user.created',
    'user.credentials.deleted',
    'user.deleted',
    'user.role.unassigned',
    'user.updated',
    'user.email_verified',
    'auth.reset_password',
  ],
  authentication: [
    'auth.device_flow.initiated',
    'auth.device_flow.denied',
    'auth.device_flow.expired',
  ],
  webhook: ['webhook.created', 'webhook.deleted', 'webhook.updated'],
}

export const webhookCategoryLabelKey = (category: string) => `category.${category}`

export const webhookTriggerLabelKey = (trigger: Schemas.WebhookTrigger) =>
  `trigger.${trigger}.label`

export const webhookTriggerDescriptionKey = (trigger: Schemas.WebhookTrigger) =>
  `trigger.${trigger}.description`

export type WebhookCategory = {
  category: string
  labelKey: string
  events: {
    key: Schemas.WebhookTrigger
    labelKey: string
    descriptionKey: string
  }[]
}

export const getWebhookCategoriesForUI = (): WebhookCategory[] => {
  return Object.entries(WEBHOOK_CATEGORIES).map(([category, triggers]) => ({
    category,
    labelKey: webhookCategoryLabelKey(category),
    events: triggers.map((trigger) => ({
      key: trigger,
      labelKey: webhookTriggerLabelKey(trigger),
      descriptionKey: webhookTriggerDescriptionKey(trigger),
    })),
  }))
}

export const getAllWebhookTriggers = (): Schemas.WebhookTrigger[] => {
  return Object.values(WEBHOOK_CATEGORIES).flat()
}
