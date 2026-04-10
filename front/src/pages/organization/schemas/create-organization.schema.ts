import { z } from 'zod'

export const createOrganizationSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  alias: z
    .string()
    .min(1, 'Alias is required')
    .regex(/^[a-z0-9_-]+$/, 'Only lowercase letters, numbers, hyphens and underscores'),
  domain: z.string().nullable().optional(),
  redirectUrl: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  enabled: z.boolean(),
})

export type CreateOrganizationSchema = z.infer<typeof createOrganizationSchema>
