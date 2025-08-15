import { z } from 'zod'

export const setCredentialPasswordSchema = z.object({
  password: z.string().min(5, { message: 'Password must be at least 5 characters long' }),
  confirmPassword: z.string().min(5, { message: 'Confirm Password must be at least 5 characters long' }),
  temporary: z.boolean(),
}).refine(data => data.password === data.confirmPassword, {
  message: 'Passwords must match',
  path: ['confirmPassword'],
})

export type SetCredentialPasswordSchema = z.infer<typeof setCredentialPasswordSchema>;
