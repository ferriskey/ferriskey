import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const updateOrganizationSchema = z.object({
  name: z.string().min(1, { error: () => translate('organization:validation.name_required') }),
  alias: z
    .string()
    .min(1, { error: () => translate('organization:validation.alias_required') })
    .regex(/^[a-z0-9_-]+$/, {
      error: () => translate('organization:validation.alias_pattern.settings'),
    }),
  domain: z.string().nullable().optional(),
  redirectUrl: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  enabled: z.boolean(),
})

export type UpdateOrganizationSchema = z.infer<typeof updateOrganizationSchema>
