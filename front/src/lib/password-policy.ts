import { z } from 'zod'
import { DEFAULT_PASSWORD_POLICY, type PublicPasswordPolicy } from '@/api/password-policy.api'
import { translate } from '@/lib/i18n'

const MAX_PASSWORD_LENGTH = 100

export type PasswordRuleCode =
  | 'too_short'
  | 'missing_uppercase'
  | 'missing_lowercase'
  | 'missing_number'
  | 'missing_special'

export interface PasswordRequirement {
  id: PasswordRuleCode
  label: string
  message: string
  isMet: (password: string) => boolean
}

const ruleText = (code: PasswordRuleCode, field: 'label' | 'message', count?: number) =>
  translate(`common:password.rule.${code}.${field}`, count === undefined ? undefined : { count })

export function passwordPolicyRequirements(policy?: PublicPasswordPolicy): PasswordRequirement[] {
  const minLength = policy?.min_length ?? DEFAULT_PASSWORD_POLICY.min_length

  const requirements: PasswordRequirement[] = [
    {
      id: 'too_short',
      label: ruleText('too_short', 'label', minLength),
      message: ruleText('too_short', 'message', minLength),
      isMet: (password) => password.length >= minLength,
    },
  ]

  if (policy?.require_uppercase) {
    requirements.push({
      id: 'missing_uppercase',
      label: ruleText('missing_uppercase', 'label'),
      message: ruleText('missing_uppercase', 'message'),
      isMet: (password) => /[A-Z]/.test(password),
    })
  }
  if (policy?.require_lowercase) {
    requirements.push({
      id: 'missing_lowercase',
      label: ruleText('missing_lowercase', 'label'),
      message: ruleText('missing_lowercase', 'message'),
      isMet: (password) => /[a-z]/.test(password),
    })
  }
  if (policy?.require_number) {
    requirements.push({
      id: 'missing_number',
      label: ruleText('missing_number', 'label'),
      message: ruleText('missing_number', 'message'),
      isMet: (password) => /[0-9]/.test(password),
    })
  }
  if (policy?.require_special) {
    requirements.push({
      id: 'missing_special',
      label: ruleText('missing_special', 'label'),
      message: ruleText('missing_special', 'message'),
      isMet: (password) => /[^A-Za-z0-9]/.test(password),
    })
  }

  return requirements
}

export function buildPasswordField(policy?: PublicPasswordPolicy) {
  return z
    .string()
    .max(MAX_PASSWORD_LENGTH, {
      error: () =>
        translate('common:password.rule.too_long.message', { total: MAX_PASSWORD_LENGTH }),
    })
    .superRefine((password, ctx) => {
      for (const requirement of passwordPolicyRequirements(policy)) {
        if (requirement.isMet(password)) continue
        ctx.addIssue({ code: 'custom', message: requirement.message })
      }
    })
}
