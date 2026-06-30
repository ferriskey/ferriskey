import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { evaluatePassword } from '../utils/password-policy'

export function buildResetPasswordSchema(policy: PublicPasswordPolicy) {
  return z
    .object({
      password: z
        .string()
        .min(1, 'Password is required')
        .superRefine((value, ctx) => {
          const result = evaluatePassword(value, policy)
          if (!result.valid) {
            ctx.addIssue({
              code: 'custom',
              message: result.unmetMessages.join(', '),
            })
          }
        }),
      confirmPassword: z.string(),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: 'Passwords do not match',
      path: ['confirmPassword'],
    })
}

export type ResetPasswordSchema = z.infer<ReturnType<typeof buildResetPasswordSchema>>
