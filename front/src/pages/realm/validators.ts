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
})

export type UpdateRealmSchema = z.infer<typeof updateRealmValidator>
export type CreateWebhookSchema = z.infer<typeof createWebhookValidator>
