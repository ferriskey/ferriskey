import { Input } from '@/components/ui/input'
import { FieldRow, Section, SwitchField } from '@/components/kit'

export interface PolicyDraft {
  min_length: number | null
  require_uppercase: boolean
  require_lowercase: boolean
  require_number: boolean
  require_special: boolean
  max_age_days: number | null
  min_entropy_bits: number | null
  forbid_common: boolean
  check_breached: boolean
}

export interface RealmPasswordPolicyTabProps {
  value: PolicyDraft
  errors: Partial<Record<keyof PolicyDraft, string>>
  onChange: (patch: Partial<PolicyDraft>) => void
}

const REQUIREMENTS: {
  key: string
  label: string
  description: string
  read: (draft: PolicyDraft) => boolean
  patch: (v: boolean) => Partial<PolicyDraft>
}[] = [
  {
    key: 'uppercase',
    label: 'Require Uppercase',
    description: 'Force users to include at least one uppercase letter.',
    read: (d) => d.require_uppercase,
    patch: (v) => ({ require_uppercase: v }),
  },
  {
    key: 'lowercase',
    label: 'Require Lowercase',
    description: 'Force users to include at least one lowercase letter.',
    read: (d) => d.require_lowercase,
    patch: (v) => ({ require_lowercase: v }),
  },
  {
    key: 'number',
    label: 'Require Number',
    description: 'Force users to include at least one numeric digit.',
    read: (d) => d.require_number,
    patch: (v) => ({ require_number: v }),
  },
  {
    key: 'special',
    label: 'Require Special Character',
    description:
      'Force users to include at least one special character (!@#$%…).',
    read: (d) => d.require_special,
    patch: (v) => ({ require_special: v }),
  },
]

function NumberField({
  id,
  value,
  unit,
  error,
  onChange,
}: {
  id: string
  value: number | null
  unit: string
  error?: string
  onChange: (v: number | null) => void
}) {
  return (
    <>
      <div className='flex max-w-[12rem] items-center gap-2'>
        <Input
          id={id}
          type='number'
          value={value ?? ''}
          onChange={(e) => onChange(e.target.value === '' ? null : Number(e.target.value))}
          className='tnum'
          aria-invalid={Boolean(error)}
        />
        <span className='shrink-0 text-xs text-neutral-500 dark:text-neutral-400'>{unit}</span>
      </div>
      {error && <p className='mt-1.5 text-xs text-fk-danger'>{error}</p>}
    </>
  )
}

export default function RealmPasswordPolicyTab({
  value,
  errors,
  onChange,
}: RealmPasswordPolicyTabProps) {
  return (
    <Section
      title='Password Policy'
      description='Rules applied to the passwords of the accounts of this realm.'
    >
      <FieldRow
        label='Minimum Length'
        description='The minimum number of characters required.'
        htmlFor='policy-min-length'
      >
        <NumberField
          id='policy-min-length'
          value={value.min_length}
          unit='characters'
          error={errors.min_length}
          onChange={(v) => onChange({ min_length: v })}
        />
      </FieldRow>

      {REQUIREMENTS.map((requirement) => (
        <FieldRow
          key={requirement.key}
          label={requirement.label}
          description={requirement.description}
        >
          <SwitchField
            checked={requirement.read(value)}
            onCheckedChange={(v) => onChange(requirement.patch(v))}
          />
        </FieldRow>
      ))}

      <FieldRow
        label='Password Expiry'
        description='Force a password change after this many days. 0 to disable.'
        htmlFor='policy-max-age'
      >
        <NumberField
          id='policy-max-age'
          value={value.max_age_days}
          unit='days'
          error={errors.max_age_days}
          onChange={(v) => onChange({ max_age_days: v })}
        />
      </FieldRow>

      <FieldRow
        label='Minimum Entropy'
        description='Reject passwords weaker than this Shannon entropy. 0 to disable — CNIL deliberation 2022-100 suggests 80.'
        htmlFor='policy-entropy'
      >
        <NumberField
          id='policy-entropy'
          value={value.min_entropy_bits}
          unit='bits'
          error={errors.min_entropy_bits}
          onChange={(v) => onChange({ min_entropy_bits: v })}
        />
      </FieldRow>

      <FieldRow
        label='Forbid Common Passwords'
        description='Reject passwords from the common-password list, and those reusing the username or the email address.'
      >
        <SwitchField
          checked={value.forbid_common}
          onCheckedChange={(v) => onChange({ forbid_common: v })}
        />
      </FieldRow>

      <FieldRow
        label='Check Breached Passwords'
        description='Reject passwords reported as breached. Without an external provider configured, this setting has no effect.'
      >
        <SwitchField
          checked={value.check_breached}
          onCheckedChange={(v) => onChange({ check_breached: v })}
        />
      </FieldRow>
    </Section>
  )
}
