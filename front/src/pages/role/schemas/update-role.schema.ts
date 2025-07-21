import { z } from 'zod'

export const updateRoleSchema = z.object({
  name: z.string().min(1, { message: 'Role name is required' }),
  description: z.string().optional(),
})

export const updateRolePermissionsSchema = z.object({
  permissions: z.array(z.string()),
})

export type UpdateRoleSchema = z.infer<typeof updateRoleSchema>
export type UpdateRolePermissionsSchema = z.infer<typeof updateRolePermissionsSchema>