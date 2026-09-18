import { Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import SaveBar from '@/components/kit/save-bar'
import { PageShell, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import RealmPasswordPolicyTab, {
  type PolicyDraft,
} from '@/pages/iam/realm/ui/realm-password-policy-tab'

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

type ConsoleTranslate = TFunction<'console'>

interface PolicyRule {
  key: keyof PolicyDraft
  label: (draft: PolicyDraft, t: ConsoleTranslate) => string
  active: (draft: PolicyDraft) => boolean
}

const ANNOUNCED: PolicyRule[] = [
  {
    key: 'min_length',
    label: (d, t) =>
      t('authentication.password_policy.rules.min_length', { count: d.min_length ?? 0 }),
    active: (d) => Boolean(d.min_length && d.min_length > 0),
  },
  {
    key: 'require_uppercase',
    label: (_, t) => t('authentication.password_policy.rules.require_uppercase'),
    active: (d) => d.require_uppercase,
  },
  {
    key: 'require_lowercase',
    label: (_, t) => t('authentication.password_policy.rules.require_lowercase'),
    active: (d) => d.require_lowercase,
  },
  {
    key: 'require_number',
    label: (_, t) => t('authentication.password_policy.rules.require_number'),
    active: (d) => d.require_number,
  },
  {
    key: 'require_special',
    label: (_, t) => t('authentication.password_policy.rules.require_special'),
    active: (d) => d.require_special,
  },
]

const SILENT: PolicyRule[] = [
  {
    key: 'min_entropy_bits',
    label: (d, t) =>
      t('authentication.password_policy.rules.min_entropy', { count: d.min_entropy_bits ?? 0 }),
    active: (d) => Boolean(d.min_entropy_bits && d.min_entropy_bits > 0),
  },
  {
    key: 'forbid_common',
    label: (_, t) => t('authentication.password_policy.rules.forbid_common'),
    active: (d) => d.forbid_common,
  },
  {
    key: 'check_breached',
    label: (_, t) => t('authentication.password_policy.rules.check_breached'),
    active: (d) => d.check_breached,
  },
]

const RULE_SEPARATOR = ', '

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
  const { t } = useTranslation('console')

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-2 h-4 w-72 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-6 h-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
      </PageShell>
    )
  }

  if (failed) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('authentication.password_policy.loading_failed.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('authentication.password_policy.loading_failed.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const announced = ANNOUNCED.filter((rule) => rule.active(value)).map((rule) =>
    rule.label(value, t)
  )
  const silent = SILENT.filter((rule) => rule.active(value)).map((rule) => rule.label(value, t))

  return (
    <PageShell>
      <div className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{t('authentication.password_policy.title')}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {t('authentication.password_policy.description')}
          </p>
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        <RealmPasswordPolicyTab value={value} errors={errors} onChange={onChange} />

        <Section
          title={t('authentication.password_policy.preview.title')}
          description={t('authentication.password_policy.preview.description')}
        >
          <div className='py-4'>
            <p className='text-xs font-medium text-neutral-500 dark:text-neutral-400'>
              {t('authentication.password_policy.preview.listed')}
            </p>
            {announced.length > 0 ? (
              <ul className='mt-1.5 space-y-1 text-[13px] text-neutral-700 dark:text-neutral-300'>
                {announced.map((rule) => (
                  <li key={rule}>{rule}</li>
                ))}
              </ul>
            ) : (
              <p className='mt-1.5 text-[13px] text-neutral-500 dark:text-neutral-400'>
                {t('authentication.password_policy.preview.none')}
              </p>
            )}

            {silent.length > 0 && (
              <div className='mt-4 flex items-start gap-2.5 rounded-sm border border-fk-line bg-neutral-50/60 px-3 py-2.5 dark:bg-fk-raised/40'>
                <Info className='mt-0.5 size-3.5 shrink-0 text-neutral-400' strokeWidth={2} />
                <p className='text-xs text-neutral-600 dark:text-neutral-400'>
                  {t('authentication.password_policy.preview.silent', {
                    rules: silent.join(RULE_SEPARATOR),
                  })}
                </p>
              </div>
            )}

            {value.max_age_days !== null && value.max_age_days > 0 && (
              <p className='mt-3 text-xs text-neutral-600 dark:text-neutral-400'>
                {t('authentication.password_policy.preview.expiry', {
                  count: value.max_age_days,
                })}
              </p>
            )}
          </div>
        </Section>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={t('authentication.password_policy.save_bar.title', { count: dirtyCount })}
        description={t('authentication.password_policy.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('authentication.password_policy.save_bar.cancel')}
        actions={[
          {
            label: isSaving
              ? t('authentication.password_policy.save_bar.saving')
              : t('authentication.password_policy.save_bar.submit'),
            onClick: onSave,
            variant: canSave && !isSaving ? 'default' : 'secondary',
          },
        ]}
      />
    </PageShell>
  )
}
