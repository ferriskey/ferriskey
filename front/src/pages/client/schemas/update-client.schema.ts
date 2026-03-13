import { z } from 'zod'

export const updateClientSchema = z.object({
  clientId: z.string().min(1, { message: 'The client ID is required' }),
  name: z.string().min(1, { message: 'The name is required' }),
  enabled: z.boolean().optional(),
  directAccessGrantsEnabled: z.boolean().optional(),
  accessTokenLifetime: z.number().min(60).max(86400).nullable().optional(),
  refreshTokenLifetime: z.number().min(300).max(2592000).nullable().optional(),
  idTokenLifetime: z.number().min(60).max(86400).nullable().optional(),
  temporaryTokenLifetime: z.number().min(60).max(86400).nullable().optional(),
})

export type UpdateClientSchema = z.infer<typeof updateClientSchema>
