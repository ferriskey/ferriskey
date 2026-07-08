import { z } from 'zod'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'

/**
 * Build the reset-password validation schema from the realm's actual password policy
 * instead of hardcoding character-class rules. Character-class / length checks are
 * validated client-side; entropy and common-password checks are enforced by the backend
 * (they need the entropy estimator / common-password list) and surface as API errors.
 */
export function buildSetCredentialPasswordSchema(policy?: PublicPasswordPolicy) {
  const minLength = policy?.min_length ?? 8

  let password = z.string().min(minLength, {
    message: `Password must be at least ${minLength} characters long`,
  })

  if (policy?.require_uppercase) {
    password = password.regex(/[A-Z]/, {
      message: 'Password must contain at least one uppercase letter',
    })
  }
  if (policy?.require_lowercase) {
    password = password.regex(/[a-z]/, {
      message: 'Password must contain at least one lowercase letter',
    })
  }
  if (policy?.require_number) {
    password = password.regex(/[0-9]/, {
      message: 'Password must contain at least one number',
    })
  }
  if (policy?.require_special) {
    password = password.regex(/[^A-Za-z0-9]/, {
      message: 'Password must contain at least one special character',
    })
  }

  return z
    .object({
      password,
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
