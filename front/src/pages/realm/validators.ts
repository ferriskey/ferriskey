import { SigningAlgorithm } from '@/api/core.interface'
import { z } from 'zod'

export const updateRealmValidator = z.object({
  name: z.string().min(1),
  default_signing_algorithm: z.nativeEnum(SigningAlgorithm),
})

export const createWebhookValidator = z.object({
  name: z.string(),
  description: z.string().optional(),
  endpoint: z.string().url().optional(),
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

export type UpdateRealmSchema = z.infer<typeof updateRealmValidator>
export type CreateWebhookSchema = z.infer<typeof createWebhookValidator>

export const updatePasswordPolicyValidator = z.object({
  min_length: z.number().min(1).max(128).nullable().optional(),
  require_uppercase: z.boolean().nullable().optional(),
  require_lowercase: z.boolean().nullable().optional(),
  require_number: z.boolean().nullable().optional(),
  require_special: z.boolean().nullable().optional(),
  max_age_days: z.number().min(0).nullable().optional(),
})

export type UpdatePasswordPolicySchema = z.infer<typeof updatePasswordPolicyValidator>
