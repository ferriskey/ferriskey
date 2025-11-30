import { z } from 'zod'

export const setCredentialPasswordSchema = z
  .object({
    password: z
      .string()
      .min(5, { message: 'Password must be at least 5 characters long' })
      .regex(/[A-Z]/, { message: 'Password must contain at least one uppercase letter' })
      .regex(/[0-9]/, { message: 'Password must contain at least one number' })
      .regex(/[^A-Za-z0-9]/, { message: 'Password must contain at least one special character' }),

    confirmPassword: z
      .string()
      .min(5, { message: 'Confirm Password must be at least 5 characters long' }),

    temporary: z.boolean(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'Passwords must match',
    path: ['confirmPassword'],
  })

export type SetCredentialPasswordSchema = z.infer<typeof setCredentialPasswordSchema>
