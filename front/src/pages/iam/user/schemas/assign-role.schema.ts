import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const assignRoleSchema = z.object({
  roleIds: z.array(z.string()).min(1, { error: () => translate('user:validation.role_required') }),
})

export type AssignRoleSchema = z.infer<typeof assignRoleSchema>
