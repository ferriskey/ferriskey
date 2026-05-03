import { z } from 'zod'

export const createUserValidator = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  email: z.union([z.string().email(), z.literal('')]).optional(),
  email_verified: z.boolean().optional(),
})

export const updateUserValidator = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  enabled: z.boolean().optional(),
  email: z.union([z.string().email(), z.literal('')]).optional(),
  email_verified: z.boolean().optional(),
  required_actions: z.array(z.string()).optional(),
})


export type CreateUserSchema = z.infer<typeof createUserValidator>
export type UpdateUserSchema = z.infer<typeof updateUserValidator>
