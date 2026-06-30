import { cn } from '@/lib/utils'
import { Check, X } from 'lucide-react'
import type { PublicPasswordPolicy } from '@/api/password-policy.api'
import { evaluatePassword, hasPolicyRules } from '../utils/password-policy'

export interface PasswordRequirementsProps {
  password: string
  policy: PublicPasswordPolicy
  className?: string
}

interface RuleRowProps {
  label: string
  met: boolean
  show: boolean
}

function RuleRow({ label, met, show }: RuleRowProps) {
  if (!show) return null
  return (
    <li className='flex items-center gap-2 text-sm'>
      {met ? (
        <Check className='h-3.5 w-3.5 shrink-0 text-green-500' />
      ) : (
        <X className='h-3.5 w-3.5 shrink-0 text-destructive' />
      )}
      <span className={cn(met ? 'text-muted-foreground line-through' : 'text-foreground')}>
        {label}
      </span>
    </li>
  )
}

export default function PasswordRequirements({
  password,
  policy,
  className,
}: PasswordRequirementsProps) {
  if (!hasPolicyRules(policy)) return null

  const eval_ = evaluatePassword(password, policy)

  return (
    <ul className={cn('space-y-1 rounded-md border border-border bg-muted/40 px-3 py-2', className)}>
      <RuleRow
        label={`At least ${policy.min_length} characters`}
        met={eval_.minLength}
        show={policy.min_length > 0}
      />
      <RuleRow
        label='At least one uppercase letter'
        met={eval_.uppercase}
        show={policy.require_uppercase}
      />
      <RuleRow
        label='At least one lowercase letter'
        met={eval_.lowercase}
        show={policy.require_lowercase}
      />
      <RuleRow
        label='At least one number'
        met={eval_.number}
        show={policy.require_number}
      />
      <RuleRow
        label='At least one special character (!@#$%^&*…)'
        met={eval_.special}
        show={policy.require_special}
      />
    </ul>
  )
}
