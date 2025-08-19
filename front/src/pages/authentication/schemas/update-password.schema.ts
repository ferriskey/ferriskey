import { z } from 'zod'

export const updatePasswordSchema = z
  .object({
    password: z.string().min(1).max(100),
    confirmPassword: z.string().min(1).max(100),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'Password must match',
    path: ['confirmPassword'],
  })

export type UpdatePasswordSchema = z.infer<typeof updatePasswordSchema>
