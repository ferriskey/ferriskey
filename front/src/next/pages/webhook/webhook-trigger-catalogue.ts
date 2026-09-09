import { getWebhookCategoriesForUI } from '@/utils/webhook-utils'

export const WEBHOOK_CATEGORIES = getWebhookCategoriesForUI()

export const WEBHOOK_TRIGGER_COUNT = WEBHOOK_CATEGORIES.reduce(
  (n, category) => n + category.events.length,
  0
)
