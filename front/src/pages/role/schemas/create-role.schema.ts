import { z } from 'zod'

export const createRoleSchema = z
  .object({
    name: z.string().min(1, { message: 'Role name is required' }),
    scope: z.enum(['realm', 'client']),
    clientId: z.string().optional(),
    description: z.string().optional(),
    permissions: z.array(z.string()),
  })
  .superRefine((value, ctx) => {
    if (value.scope === 'client' && (!value.clientId || value.clientId.trim().length === 0)) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['clientId'],
        message: 'Client ID is required for client roles',
      })
    }
  })

export type CreateRoleSchema = z.infer<typeof createRoleSchema>
