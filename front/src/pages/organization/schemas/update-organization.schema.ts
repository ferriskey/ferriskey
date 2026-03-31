import { z } from 'zod'

export const updateOrganizationSchema = z.object({
  name: z.string().min(1, { message: 'Name is required' }),
  alias: z
    .string()
    .min(1, { message: 'Alias is required' })
    .regex(/^[a-z0-9_-]+$/, {
      message: 'Alias may only contain lowercase letters, digits, hyphens, or underscores',
    }),
  domain: z.string().nullable().optional(),
  redirectUrl: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  enabled: z.boolean(),
})

export type UpdateOrganizationSchema = z.infer<typeof updateOrganizationSchema>
