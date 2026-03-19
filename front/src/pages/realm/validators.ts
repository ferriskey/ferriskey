import { SigningAlgorithm } from '@/api/core.interface'
import { z } from 'zod'

export const updateRealmValidator = z.object({
  name: z.string().min(1),
  default_signing_algorithm: z.nativeEnum(SigningAlgorithm),
})

export const createWebhookValidator = z.object({
  name: z.string().min(1),
  description: z.string().optional(),
  endpoint: z.union([z.string().url(), z.literal('')]).optional(),
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
