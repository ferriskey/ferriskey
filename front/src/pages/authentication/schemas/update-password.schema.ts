import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { buildPasswordField } from '@/lib/password-policy'

/**
 * The forced-rotation form (a user landing here has a temporary password).
 * Rules come from the realm's public password policy so the screen refuses
 * what the backend would refuse, instead of letting the user discover it
 * one round-trip later.
 */
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

// Default schema (no policy) — kept so the inferred type stays stable across the app.
export const updatePasswordSchema = buildUpdatePasswordSchema()

export type UpdatePasswordSchema = z.infer<typeof updatePasswordSchema>
