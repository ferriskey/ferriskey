import { Info } from 'lucide-react'
import SaveBar from '@/components/kit/save-bar'
import { Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import RealmPasswordPolicyTab, {
  type PolicyDraft,
} from '@/next/pages/iam/realm/ui/realm-password-policy-tab'

export interface PagePasswordPolicyProps {
  value: PolicyDraft
  errors: Partial<Record<keyof PolicyDraft, string>>
  isLoading: boolean
  failed: boolean
  dirtyCount: number
  canSave: boolean
  isSaving: boolean
  onChange: (patch: Partial<PolicyDraft>) => void
  onDiscard: () => void
  onSave: () => void
}

const ANNOUNCED: {
  key: keyof PolicyDraft
  label: (draft: PolicyDraft) => string
  active: (draft: PolicyDraft) => boolean
}[] = [
  {
    key: 'min_length',
    label: (d) => `At least ${d.min_length ?? 0} characters`,
    active: (d) => Boolean(d.min_length && d.min_length > 0),
  },
  {
    key: 'require_uppercase',
    label: () => 'One uppercase letter',
    active: (d) => d.require_uppercase,
  },
  {
    key: 'require_lowercase',
    label: () => 'One lowercase letter',
    active: (d) => d.require_lowercase,
  },
  { key: 'require_number', label: () => 'One digit', active: (d) => d.require_number },
  {
    key: 'require_special',
    label: () => 'One special character',
    active: (d) => d.require_special,
  },
]

const SILENT: {
  key: keyof PolicyDraft
  label: (draft: PolicyDraft) => string
  active: (draft: PolicyDraft) => boolean
}[] = [
  {
    key: 'min_entropy_bits',
    label: (d) => `${d.min_entropy_bits} bits of entropy`,
    active: (d) => Boolean(d.min_entropy_bits && d.min_entropy_bits > 0),
  },
  {
    key: 'forbid_common',
    label: () => 'No common password, username or email address',
    active: (d) => d.forbid_common,
  },
  {
    key: 'check_breached',
    label: () => 'No password reported as breached',
    active: (d) => d.check_breached,
  },
]

export default function PagePasswordPolicy({
  value,
  errors,
  isLoading,
  failed,
  dirtyCount,
  canSave,
  isSaving,
  onChange,
  onDiscard,
  onSave,
}: PagePasswordPolicyProps) {
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-2 h-4 w-72 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-6 h-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
      </div>
    )
  }

  if (failed) {
    return (
      <div className={container}>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            Failed to load the password policy
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            This realm has no password policy the console can read.
          </p>
        </div>
      </div>
    )
  }

  const announced = ANNOUNCED.filter((rule) => rule.active(value)).map((rule) =>
    rule.label(value)
  )
  const silent = SILENT.filter((rule) => rule.active(value)).map((rule) => rule.label(value))

  return (
    <div className={container}>
      <div className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Password policy</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            The rules a password of this realm has to satisfy, at sign-up and at every reset.
          </p>
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        <RealmPasswordPolicyTab value={value} errors={errors} onChange={onChange} />

        <Section
          title='What the person sees'
          description='The sign-up and reset pages receive five of these rules and list them next to the field. The rest are checked on submit only.'
        >
          <div className='py-4'>
            <p className='text-xs font-medium text-neutral-500 dark:text-neutral-400'>
              Listed on the page
            </p>
            {announced.length > 0 ? (
              <ul className='mt-1.5 space-y-1 text-[13px] text-neutral-700 dark:text-neutral-300'>
                {announced.map((rule) => (
                  <li key={rule}>{rule}</li>
                ))}
              </ul>
            ) : (
              <p className='mt-1.5 text-[13px] text-neutral-500 dark:text-neutral-400'>
                Nothing — the page asks for a password with no stated requirement.
              </p>
            )}

            {silent.length > 0 && (
              <div className='mt-4 flex items-start gap-2.5 rounded-sm border border-fk-line bg-neutral-50/60 px-3 py-2.5 dark:bg-fk-raised/40'>
                <Info className='mt-0.5 size-3.5 shrink-0 text-neutral-400' strokeWidth={2} />
                <p className='text-xs text-neutral-600 dark:text-neutral-400'>
                  Enforced but never listed: {silent.join(', ')}. The person only learns about
                  them from the rejection message after they submitted a password.
                </p>
              </div>
            )}

            {value.max_age_days !== null && value.max_age_days > 0 && (
              <p className='mt-3 text-xs text-neutral-600 dark:text-neutral-400'>
                Every password expires after {value.max_age_days} days; the person is sent to the
                reset screen at their first sign-in past that date.
              </p>
            )}
          </div>
        </Section>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Existing passwords are not re-checked; the new rules apply at the next sign-up or reset.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[
          {
            label: isSaving ? 'Saving...' : 'Save changes',
            onClick: onSave,
            variant: canSave && !isSaving ? 'default' : 'secondary',
          },
        ]}
      />
    </div>
  )
}
