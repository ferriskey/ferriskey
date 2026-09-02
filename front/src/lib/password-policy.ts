import { z } from 'zod'
import { DEFAULT_PASSWORD_POLICY, type PublicPasswordPolicy } from '@/api/password-policy.api'

/**
 * One rule of the realm's password policy, in the two shapes the UI needs:
 * a short `label` for the live checklist shown under the input, and a full
 * `message` for the validation error shown when the form is submitted.
 *
 * Both the zod schema and the checklist are derived from this same list, so
 * they can never disagree about what the realm actually requires.
 */
export interface PasswordRequirement {
  id: string
  label: string
  message: string
  isMet: (password: string) => boolean
}

/**
 * The rules we can check in the browser. Entropy, common-password and breach
 * checks are deliberately absent: they need the backend's estimator and word
 * lists, so they are enforced server-side and surface as API errors on the
 * field.
 */
export function passwordPolicyRequirements(
  policy?: PublicPasswordPolicy
): PasswordRequirement[] {
  const minLength = policy?.min_length ?? DEFAULT_PASSWORD_POLICY.min_length

  const requirements: PasswordRequirement[] = [
    {
      id: 'length',
      label: `At least ${minLength} characters`,
      message: `Password must be at least ${minLength} characters long`,
      isMet: (password) => password.length >= minLength,
    },
  ]

  if (policy?.require_uppercase) {
    requirements.push({
      id: 'uppercase',
      label: 'One uppercase letter',
      message: 'Password must contain at least one uppercase letter',
      isMet: (password) => /[A-Z]/.test(password),
    })
  }
  if (policy?.require_lowercase) {
    requirements.push({
      id: 'lowercase',
      label: 'One lowercase letter',
      message: 'Password must contain at least one lowercase letter',
      isMet: (password) => /[a-z]/.test(password),
    })
  }
  if (policy?.require_number) {
    requirements.push({
      id: 'number',
      label: 'One number',
      message: 'Password must contain at least one number',
      isMet: (password) => /[0-9]/.test(password),
    })
  }
  if (policy?.require_special) {
    requirements.push({
      id: 'special',
      label: 'One special character',
      message: 'Password must contain at least one special character',
      isMet: (password) => /[^A-Za-z0-9]/.test(password),
    })
  }

  return requirements
}

/**
 * Build the zod field for a new password from the realm's policy. Every
 * unmet requirement is reported, not just the first, so a user who is two
 * rules away learns both at once.
 */
export function buildPasswordField(policy?: PublicPasswordPolicy) {
  const requirements = passwordPolicyRequirements(policy)

  // The upper bound is not a policy rule — it is a guard against a
  // megabyte-sized string reaching the hasher — so it stays out of the
  // requirement list the checklist renders.
  return z
    .string()
    .max(100, { message: 'Password must be at most 100 characters long' })
    .superRefine((password, ctx) => {
      for (const requirement of requirements) {
        if (requirement.isMet(password)) continue
        ctx.addIssue({ code: 'custom', message: requirement.message })
      }
    })
}
