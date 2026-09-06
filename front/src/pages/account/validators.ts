import { z } from 'zod'

export const updateOwnProfileValidator = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  email: z.union([z.string().email(), z.literal('')]).optional(),
})

export type UpdateOwnProfileSchema = z.infer<typeof updateOwnProfileValidator>
