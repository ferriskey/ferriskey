import { SigningAlgorithm } from '@/api/core.interface'
import { translate } from '@/lib/i18n'
import { z } from 'zod'

const DISPLAY_NAME_MAX_LENGTH = 255

export const updateRealmValidator = z.object({
  name: z.string().min(1),
  display_name: z
    .string()
    .max(DISPLAY_NAME_MAX_LENGTH, {
      error: () =>
        translate('realm:validation.display_name_max', { max: DISPLAY_NAME_MAX_LENGTH }),
    })
    .optional(),
  default_signing_algorithm: z.nativeEnum(SigningAlgorithm),
})

export const createWebhookValidator = z.object({
  name: z.string().min(1, { error: () => translate('webhook:validation.name_required') }),
  description: z.string().optional(),
  endpoint: z
    .union([
      z.string().url({ error: () => translate('webhook:validation.endpoint_invalid') }),
      z.literal(''),
    ])
    .optional(),
  subscribers: z.array(z.string()),
  headers: z
    .array(
      z.object({
        key: z.string(),
        value: z.string(),
      })
    )
    .optional(),
})

export const updateWebhookValidator = createWebhookValidator

export type UpdateRealmSchema = z.infer<typeof updateRealmValidator>
export type CreateWebhookSchema = z.infer<typeof createWebhookValidator>
export type UpdateWebhookSchema = z.infer<typeof updateWebhookValidator>

export const updatePasswordPolicyValidator = z.object({
  min_length: z.number().min(1).max(128).nullable().optional(),
  require_uppercase: z.boolean().nullable().optional(),
  require_lowercase: z.boolean().nullable().optional(),
  require_number: z.boolean().nullable().optional(),
  require_special: z.boolean().nullable().optional(),
  max_age_days: z.number().min(0).nullable().optional(),
  min_entropy_bits: z.number().min(0).max(256).nullable().optional(),
  forbid_common: z.boolean().nullable().optional(),
  check_breached: z.boolean().nullable().optional(),
})

export type UpdatePasswordPolicySchema = z.infer<typeof updatePasswordPolicyValidator>
