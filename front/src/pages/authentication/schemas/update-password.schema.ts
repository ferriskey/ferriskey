import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { buildPasswordField } from '@/lib/password-policy'

export function buildUpdatePasswordSchema(policy?: PublicPasswordPolicy) {
  return z
    .object({
      password: buildPasswordField(policy),
      confirmPassword: z.string().min(1, { message: 'Confirm your new password' }),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: 'Password must match',
      path: ['confirmPassword'],
    })
}

export const updatePasswordSchema = buildUpdatePasswordSchema()

export type UpdatePasswordSchema = z.infer<typeof updatePasswordSchema>
