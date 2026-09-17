import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { translate } from '@/lib/i18n'
import { evaluatePassword } from '../utils/password-policy'
import { AUTH_NAMESPACE } from '../constants'

export function buildResetPasswordSchema(policy: PublicPasswordPolicy) {
  return z
    .object({
      password: z
        .string()
        .min(1, { error: () => translate(`${AUTH_NAMESPACE}:validation.password_required`) })
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
      error: () => translate(`${AUTH_NAMESPACE}:validation.passwords_mismatch`),
      path: ['confirmPassword'],
    })
}

export type ResetPasswordSchema = z.infer<ReturnType<typeof buildResetPasswordSchema>>
