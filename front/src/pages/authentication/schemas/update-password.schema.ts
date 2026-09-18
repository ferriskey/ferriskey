import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { buildPasswordField } from '@/lib/password-policy'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

export function buildUpdatePasswordSchema(policy?: PublicPasswordPolicy) {
  return z
    .object({
      password: buildPasswordField(policy),
      confirmPassword: z.string().min(1, {
        error: () => translate(`${AUTH_NAMESPACE}:validation.password_confirm_new_required`),
      }),
    })
    .refine((data) => data.password === data.confirmPassword, {
      error: () => translate(`${AUTH_NAMESPACE}:validation.password_must_match`),
      path: ['confirmPassword'],
    })
}

export const updatePasswordSchema = buildUpdatePasswordSchema()

export type UpdatePasswordSchema = z.infer<typeof updatePasswordSchema>
