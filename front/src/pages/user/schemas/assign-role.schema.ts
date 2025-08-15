import { z } from 'zod'

export const assignRoleSchema = z.object({
  roleIds: z.array(z.string()).min(1, { message: 'At least one role must be selected' }),
})

export type AssignRoleSchema = z.infer<typeof assignRoleSchema>
