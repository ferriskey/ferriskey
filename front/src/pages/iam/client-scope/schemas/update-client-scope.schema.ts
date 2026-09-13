import { z } from 'zod'

export const updateClientScopeSchema = z.object({
  name: z.string().min(1, { message: 'The client scope name is required' }),
  description: z.string().optional(),
  scopeType: z.enum(['optional', 'default']),
})

export type UpdateClientScopeSchema = z.infer<typeof updateClientScopeSchema>
