import { z } from 'zod'
import { translate } from '@/lib/i18n'

export const updateRoleSchema = z.object({
  name: z.string().min(1, { error: () => translate('role:validation.name_required') }),
  description: z.string().optional(),
})

export const updateRolePermissionsSchema = z.object({
  permissions: z.array(z.string()),
})

export type UpdateRoleSchema = z.infer<typeof updateRoleSchema>
export type UpdateRolePermissionsSchema = z.infer<typeof updateRolePermissionsSchema>
