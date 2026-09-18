import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { REALM_NAMESPACE } from '../realm-namespace'

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
  labelKey: string
  read: (draft: PolicyDraft) => boolean
  patch: (v: boolean) => Partial<PolicyDraft>
}[] = [
  {
    key: 'uppercase',
    labelKey: 'policy.requirements.uppercase',
    read: (d) => d.require_uppercase,
    patch: (v) => ({ require_uppercase: v }),
  },
  {
    key: 'lowercase',
    labelKey: 'policy.requirements.lowercase',
    read: (d) => d.require_lowercase,
    patch: (v) => ({ require_lowercase: v }),
  },
  {
    key: 'number',
    labelKey: 'policy.requirements.number',
    read: (d) => d.require_number,
    patch: (v) => ({ require_number: v }),
  },
  {
    key: 'special',
    labelKey: 'policy.requirements.special',
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
  const { t } = useTranslation(REALM_NAMESPACE)

  return (
    <Section title={t('policy.title')} description={t('policy.description')}>
      <FieldRow
        label={t('policy.min_length.label')}
        description={t('policy.min_length.description')}
        htmlFor='policy-min-length'
      >
        <NumberField
          id='policy-min-length'
          value={value.min_length}
          unit={t('policy.min_length.unit', { count: value.min_length ?? 0 })}
          error={errors.min_length}
          onChange={(v) => onChange({ min_length: v })}
        />
      </FieldRow>

      {REQUIREMENTS.map((requirement) => (
        <FieldRow
          key={requirement.key}
          label={t(`${requirement.labelKey}.label`)}
          description={t(`${requirement.labelKey}.description`)}
        >
          <SwitchField
            checked={requirement.read(value)}
            onCheckedChange={(v) => onChange(requirement.patch(v))}
          />
        </FieldRow>
      ))}

      <FieldRow
        label={t('policy.max_age.label')}
        description={t('policy.max_age.description')}
        htmlFor='policy-max-age'
      >
        <NumberField
          id='policy-max-age'
          value={value.max_age_days}
          unit={t('policy.max_age.unit', { count: value.max_age_days ?? 0 })}
          error={errors.max_age_days}
          onChange={(v) => onChange({ max_age_days: v })}
        />
      </FieldRow>

      <FieldRow
        label={t('policy.entropy.label')}
        description={t('policy.entropy.description')}
        htmlFor='policy-entropy'
      >
        <NumberField
          id='policy-entropy'
          value={value.min_entropy_bits}
          unit={t('policy.entropy.unit', { count: value.min_entropy_bits ?? 0 })}
          error={errors.min_entropy_bits}
          onChange={(v) => onChange({ min_entropy_bits: v })}
        />
      </FieldRow>

      <FieldRow
        label={t('policy.forbid_common.label')}
        description={t('policy.forbid_common.description')}
      >
        <SwitchField
          checked={value.forbid_common}
          onCheckedChange={(v) => onChange({ forbid_common: v })}
        />
      </FieldRow>

      <FieldRow
        label={t('policy.check_breached.label')}
        description={t('policy.check_breached.description')}
      >
        <SwitchField
          checked={value.check_breached}
          onCheckedChange={(v) => onChange({ check_breached: v })}
        />
      </FieldRow>
    </Section>
  )
}
