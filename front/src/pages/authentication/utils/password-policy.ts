import type { PublicPasswordPolicy } from '@/api/password-policy.api'

/**
 * The special character set the backend validates against.
 * Must stay in sync with the Rust backend's SPECIAL_CHARS constant.
 */
export const SPECIAL_CHARS = '!@#$%^&*()_+-=[]{}|;\':",./<>?'

export interface PasswordEvaluation {
  minLength: boolean
  uppercase: boolean
  lowercase: boolean
  number: boolean
  special: boolean
  /** True only when every rule required by the policy passes */
  valid: boolean
  /** Human-readable messages for each unmet rule */
  unmetMessages: string[]
}

export function evaluatePassword(
  password: string,
  policy: PublicPasswordPolicy,
): PasswordEvaluation {
  const minLength = password.length >= policy.min_length
  const uppercase = !policy.require_uppercase || /[A-Z]/.test(password)
  const lowercase = !policy.require_lowercase || /[a-z]/.test(password)
  const number = !policy.require_number || /[0-9]/.test(password)
  const special =
    !policy.require_special || [...password].some((ch) => SPECIAL_CHARS.includes(ch))

  const unmetMessages: string[] = []
  if (!minLength) unmetMessages.push(`At least ${policy.min_length} characters`)
  if (!uppercase) unmetMessages.push('At least one uppercase letter')
  if (!lowercase) unmetMessages.push('At least one lowercase letter')
  if (!number) unmetMessages.push('At least one number')
  if (!special) unmetMessages.push('At least one special character')

  const valid = minLength && uppercase && lowercase && number && special

  return { minLength, uppercase, lowercase, number, special, valid, unmetMessages }
}

/**
 * Returns true when the policy has at least one rule that should be displayed
 * (i.e. it is stricter than the empty/default state).
 */
export function hasPolicyRules(policy: PublicPasswordPolicy): boolean {
  return (
    policy.min_length > 0 ||
    policy.require_uppercase ||
    policy.require_lowercase ||
    policy.require_number ||
    policy.require_special
  )
}
