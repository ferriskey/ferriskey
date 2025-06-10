import { z } from 'zod'

export const updateRoleSchema = z.object({
  name: z.string().min(1, { message: 'Role name is required' }),
  description: z.string().optional(),
})

export type UpdateRoleSchema = z.infer<typeof updateRoleSchema>
