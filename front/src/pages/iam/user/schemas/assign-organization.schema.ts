import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const assignOrganizationSchema = z.object({
  organizationIds: z
    .array(z.string())
    .min(1, { error: () => translate('user:validation.organization_required') }),
})

export type AssignOrganizationSchema = z.infer<typeof assignOrganizationSchema>
