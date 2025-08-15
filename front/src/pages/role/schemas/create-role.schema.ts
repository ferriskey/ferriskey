import { z } from 'zod'

export const createRoleSchema = z.object({
  name: z.string().min(1, { message: 'Role name is required' }),
  clientId: z.string().min(1, { message: 'Client ID is required' }),
  description: z.string().optional(),
  permissions: z.array(z.string()),
})

export type CreateRoleSchema = z.infer<typeof createRoleSchema>
