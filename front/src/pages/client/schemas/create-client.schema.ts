import { z } from 'zod'

export const createClientSchema = z.object({
  clientId: z.string().min(1, { message: "The client ID is required" }),
  name: z.string().min(1, { message: "The name is required" }),
  enabled: z.boolean().optional(),
  clientAuthentication: z.boolean().optional(),
  protocol: z.string().optional(),
})

export type CreateClientSchema = z.infer<typeof createClientSchema>
