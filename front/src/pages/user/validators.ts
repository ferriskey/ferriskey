import { z } from 'zod'

export const createUserValidator = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().min(1, 'First name is required'),
  lastname: z.string().min(1, 'Last name is required'),
  email: z.string().email().min(1, 'Email is required'),
  email_verified: z.boolean().optional(),
})

export const updateUserValidator = z.object({
  username: z.string().min(1, 'Username is required'),
  firstname: z.string().min(1, 'Firstname is required'),
  lastname: z.string().min(1, 'Lastname is required'),
  enabled: z.boolean().optional(),
  email: z.string().email().min(1, 'Email is required'),
  email_verified: z.boolean().optional(),
  required_actions: z.array(z.string()).optional(),
})


export type CreateUserSchema = z.infer<typeof createUserValidator>
export type UpdateUserSchema = z.infer<typeof updateUserValidator>
