import { AlertTriangle, Fingerprint, KeyRound, Link2, Mail, ShieldCheck, User } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, OrderedChoiceCards, PageShell, Section, SwitchField } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import LoginAlias = Schemas.LoginAlias

export interface SignInDraft {
  loginAliases: LoginAlias[]
  passkey: boolean
  magicLink: boolean
  magicLinkTtl: number
  requireMfa: boolean
  userRegistration: boolean
  emailVerification: boolean
  forgotPassword: boolean
  rememberMe: boolean
  lockoutThreshold: number
  lockoutDuration: number
}

export type SignInErrors = Partial<Record<keyof SignInDraft, string>>

export interface PageSignInMethodsProps {
  value: SignInDraft
  errors: SignInErrors
  isLoading: boolean
  notFound: boolean
  smtpConfigured: boolean
  dirtyCount: number
  canSave: boolean
  isSaving: boolean
  onChange: (patch: Partial<SignInDraft>) => void
  onDiscard: () => void
  onSave: () => void
}

type ConsoleTranslate = TFunction<'console'>

const aliasChoices = (t: ConsoleTranslate): Choice<LoginAlias>[] => [
  {
    value: 'username',
    label: t('authentication.sign_in.identifiers.username.label'),
    description: t('authentication.sign_in.identifiers.username.description'),
    icon: User,
  },
  {
    value: 'email',
    label: t('authentication.sign_in.identifiers.email.label'),
    description: t('authentication.sign_in.identifiers.email.description'),
    icon: Mail,
  },
]

interface Capability {
  key: string
  label: string
  description: string
  read: (draft: SignInDraft) => boolean
  patch: (v: boolean) => Partial<SignInDraft>
}

const methods = (t: ConsoleTranslate): Capability[] => [
  {
    key: 'passkey',
    label: t('authentication.sign_in.passwordless.passkey_label'),
    description: t('authentication.sign_in.passwordless.passkey_description'),
    read: (d) => d.passkey,
    patch: (v) => ({ passkey: v }),
  },
  {
    key: 'magic-link',
    label: t('authentication.sign_in.passwordless.magic_link_label'),
    description: t('authentication.sign_in.passwordless.magic_link_description'),
    read: (d) => d.magicLink,
    patch: (v) => ({ magicLink: v }),
  },
]

const access = (t: ConsoleTranslate): Capability[] => [
  {
    key: 'user-registration',
    label: t('authentication.sign_in.access.registration_label'),
    description: t('authentication.sign_in.access.registration_description'),
    read: (d) => d.userRegistration,
    patch: (v) => ({ userRegistration: v }),
  },
  {
    key: 'email-verification',
    label: t('authentication.sign_in.access.email_verification_label'),
    description: t('authentication.sign_in.access.email_verification_description'),
    read: (d) => d.emailVerification,
    patch: (v) => ({ emailVerification: v }),
  },
  {
    key: 'forgot-password',
    label: t('authentication.sign_in.access.forgot_password_label'),
    description: t('authentication.sign_in.access.forgot_password_description'),
    read: (d) => d.forgotPassword,
    patch: (v) => ({ forgotPassword: v }),
  },
  {
    key: 'remember-me',
    label: t('authentication.sign_in.access.remember_me_label'),
    description: t('authentication.sign_in.access.remember_me_description'),
    read: (d) => d.rememberMe,
    patch: (v) => ({ rememberMe: v }),
  },
]

function NumberField({
  id,
  value,
  unit,
  min,
  error,
  onChange,
}: {
  id: string
  value: number
  unit: string
  min: number
  error?: string
  onChange: (v: number) => void
}) {
  return (
    <>
      <div className='flex max-w-[12rem] items-center gap-2'>
        <Input
          id={id}
          type='number'
          min={min}
          value={value}
          onChange={(e) => onChange(e.target.value === '' ? min : Number(e.target.value))}
          className='tnum'
          aria-invalid={Boolean(error)}
        />
        <span className='shrink-0 text-xs text-neutral-500 dark:text-neutral-400'>{unit}</span>
      </div>
      {error && <p className='mt-1.5 text-xs text-fk-danger'>{error}</p>}
    </>
  )
}

function CapabilityRows({
  capabilities,
  value,
  onChange,
}: {
  capabilities: Capability[]
  value: SignInDraft
  onChange: (patch: Partial<SignInDraft>) => void
}) {
  return (
    <>
      {capabilities.map((capability) => (
        <FieldRow
          key={capability.key}
          label={capability.label}
          description={capability.description}
        >
          <SwitchField
            checked={capability.read(value)}
            onCheckedChange={(v) => onChange(capability.patch(v))}
          />
        </FieldRow>
      ))}
    </>
  )
}

export default function PageSignInMethods({
  value,
  errors,
  isLoading,
  notFound,
  smtpConfigured,
  dirtyCount,
  canSave,
  isSaving,
  onChange,
  onDiscard,
  onSave,
}: PageSignInMethodsProps) {
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

  if (notFound) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('authentication.sign_in.loading_failed.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('authentication.sign_in.loading_failed.description')}
          </p>
        </div>
      </PageShell>
    )
  }

  const emailDependent = value.magicLink || value.emailVerification || value.forgotPassword

  return (
    <PageShell>
      <div className={cn('flex flex-wrap items-start justify-between gap-3', tokens.header.spacing)}>
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{t('authentication.sign_in.title')}</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            {t('authentication.sign_in.description')}
          </p>
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        {emailDependent && !smtpConfigured && (
          <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
            <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
            <p className='text-xs text-neutral-700 dark:text-neutral-300'>
              {t('authentication.sign_in.smtp_warning')}
            </p>
          </div>
        )}

        <Section
          title={t('authentication.sign_in.identifiers.title')}
          description={t('authentication.sign_in.identifiers.description')}
        >
          <FieldRow
            label={t('authentication.sign_in.identifiers.label')}
            description={t('authentication.sign_in.identifiers.description_row')}
          >
            <OrderedChoiceCards
              label={t('authentication.sign_in.identifiers.label')}
              value={value.loginAliases}
              onChange={(next) => onChange({ loginAliases: next })}
              options={aliasChoices(t)}
              minSelectedReason={t('authentication.sign_in.identifiers.min_selected')}
            />
            {errors.loginAliases && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.loginAliases}</p>
            )}
          </FieldRow>
        </Section>

        <Section
          title={t('authentication.sign_in.passwordless.title')}
          description={t('authentication.sign_in.passwordless.description')}
        >
          <CapabilityRows capabilities={methods(t)} value={value} onChange={onChange} />

          {value.magicLink && (
            <FieldRow
              label={t('authentication.sign_in.passwordless.magic_link_ttl_label')}
              description={t('authentication.sign_in.passwordless.magic_link_ttl_description')}
              htmlFor='sign-in-magic-link-ttl'
            >
              <NumberField
                id='sign-in-magic-link-ttl'
                value={value.magicLinkTtl}
                unit={t('authentication.sign_in.passwordless.minutes')}
                min={1}
                error={errors.magicLinkTtl}
                onChange={(v) => onChange({ magicLinkTtl: v })}
              />
            </FieldRow>
          )}
        </Section>

        <Section
          title={t('authentication.sign_in.mfa.title')}
          description={t('authentication.sign_in.mfa.description')}
        >
          <FieldRow
            label={t('authentication.sign_in.mfa.label')}
            description={t('authentication.sign_in.mfa.description_row')}
          >
            <SwitchField
              checked={value.requireMfa}
              onCheckedChange={(v) => onChange({ requireMfa: v })}
            />
          </FieldRow>
        </Section>

        <Section
          title={t('authentication.sign_in.access.title')}
          description={t('authentication.sign_in.access.description')}
        >
          <CapabilityRows capabilities={access(t)} value={value} onChange={onChange} />
        </Section>

        <Section
          title={t('authentication.sign_in.protection.title')}
          description={t('authentication.sign_in.protection.description')}
        >
          <FieldRow
            label={t('authentication.sign_in.protection.threshold_label')}
            description={t('authentication.sign_in.protection.threshold_description')}
            htmlFor='sign-in-lockout-threshold'
          >
            <NumberField
              id='sign-in-lockout-threshold'
              value={value.lockoutThreshold}
              unit={t('authentication.sign_in.protection.attempts')}
              min={0}
              error={errors.lockoutThreshold}
              onChange={(v) => onChange({ lockoutThreshold: v })}
            />
          </FieldRow>

          <FieldRow
            label={t('authentication.sign_in.protection.duration_label')}
            description={t('authentication.sign_in.protection.duration_description')}
            htmlFor='sign-in-lockout-duration'
          >
            <NumberField
              id='sign-in-lockout-duration'
              value={value.lockoutDuration}
              unit={t('authentication.sign_in.protection.seconds')}
              min={0}
              error={errors.lockoutDuration}
              onChange={(v) => onChange({ lockoutDuration: v })}
            />
          </FieldRow>
        </Section>

        <div className='flex flex-wrap gap-x-6 gap-y-1 text-xs text-neutral-400 dark:text-neutral-500'>
          <span className='inline-flex items-center gap-1.5'>
            <Fingerprint className='size-3.5' strokeWidth={1.75} />
            {value.passkey
              ? t('authentication.sign_in.summary.passkey_on')
              : t('authentication.sign_in.summary.passkey_off')}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <Link2 className='size-3.5' strokeWidth={1.75} />
            {value.magicLink
              ? t('authentication.sign_in.summary.magic_link_on')
              : t('authentication.sign_in.summary.magic_link_off')}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <ShieldCheck className='size-3.5' strokeWidth={1.75} />
            {value.requireMfa
              ? t('authentication.sign_in.summary.mfa_on')
              : t('authentication.sign_in.summary.mfa_off')}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <KeyRound className='size-3.5' strokeWidth={1.75} />
            {value.lockoutThreshold > 0
              ? t('authentication.sign_in.summary.lockout_on', {
                  count: value.lockoutThreshold,
                })
              : t('authentication.sign_in.summary.lockout_off')}
          </span>
        </div>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={t('authentication.sign_in.save_bar.title', { count: dirtyCount })}
        description={t('authentication.sign_in.save_bar.description')}
        onCancel={onDiscard}
        cancelLabel={t('authentication.sign_in.save_bar.cancel')}
        actions={[
          {
            label: isSaving
              ? t('authentication.sign_in.save_bar.saving')
              : t('authentication.sign_in.save_bar.submit'),
            onClick: onSave,
            variant: canSave && !isSaving ? 'default' : 'secondary',
          },
        ]}
      />
    </PageShell>
  )
}
