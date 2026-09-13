import { z } from 'zod'

export const createClientScopeSchema = z.object({
  name: z.string().min(1, { message: 'The client scope name is required' }),
  description: z.string().optional(),
  protocol: z.literal('openid-connect'),
  scopeType: z.enum(['optional', 'default']),
})

export type CreateClientScopeSchema = z.infer<typeof createClientScopeSchema>
