import { z } from 'zod'

export const updateClientSchema = z.object({
  clientId: z.string().min(1, { message: 'The client ID is required' }),
  name: z.string().min(1, { message: 'The name is required' }),
  enabled: z.boolean().optional(),
  directAccessGrantsEnabled: z.boolean().optional(),
  accessTokenLifetime: z.number().nullable().optional(),
  refreshTokenLifetime: z.number().nullable().optional(),
  idTokenLifetime: z.number().nullable().optional(),
  temporaryTokenLifetime: z.number().nullable().optional(),
})

export type UpdateClientSchema = z.infer<typeof updateClientSchema>
