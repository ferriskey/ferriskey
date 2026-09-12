import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { buildPasswordField } from '@/lib/password-policy'

/**
 * Build the reset-password validation schema from the realm's actual password policy
 * instead of hardcoding character-class rules. Character-class / length checks are
 * validated client-side; entropy and common-password checks are enforced by the backend
 * (they need the entropy estimator / common-password list) and surface as API errors.
 */
export function buildSetCredentialPasswordSchema(policy?: PublicPasswordPolicy) {
  return z
    .object({
      password: buildPasswordField(policy),
      confirmPassword: z.string(),
      temporary: z.boolean(),
    })
    .refine((data) => data.password === data.confirmPassword, {
      message: 'Passwords must match',
      path: ['confirmPassword'],
    })
}

// Default schema (no policy) — kept so the inferred type stays stable across the app.
export const setCredentialPasswordSchema = buildSetCredentialPasswordSchema()

export type SetCredentialPasswordSchema = z.infer<typeof setCredentialPasswordSchema>
