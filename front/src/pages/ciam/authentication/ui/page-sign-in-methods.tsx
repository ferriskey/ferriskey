import { AlertTriangle, Fingerprint, KeyRound, Link2, Mail, ShieldCheck, User } from 'lucide-react'
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

const ALIAS_CHOICES: Choice<LoginAlias>[] = [
  {
    value: 'username',
    label: 'Username',
    description: 'The name the person chose when the account was opened.',
    icon: User,
  },
  {
    value: 'email',
    label: 'Email',
    description: 'The email address recorded on the account.',
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

const METHODS: Capability[] = [
  {
    key: 'passkey',
    label: 'Passkey',
    description:
      'The sign-in page offers a passkey button. The person unlocks their device with a fingerprint, a face or a security key and never types a password.',
    read: (d) => d.passkey,
    patch: (v) => ({ passkey: v }),
  },
  {
    key: 'magic-link',
    label: 'Magic link',
    description:
      'The person types their email address and receives a one-time sign-in link. Opening the link signs them in without a password.',
    read: (d) => d.magicLink,
    patch: (v) => ({ magicLink: v }),
  },
]

const ACCESS: Capability[] = [
  {
    key: 'user-registration',
    label: 'Self-service sign-up',
    description:
      'A "Create an account" link appears on the sign-in page and anyone can open an account without an operator.',
    read: (d) => d.userRegistration,
    patch: (v) => ({ userRegistration: v }),
  },
  {
    key: 'email-verification',
    label: 'Email verification',
    description:
      'A new account has to confirm its email address from a message before its first sign-in completes.',
    read: (d) => d.emailVerification,
    patch: (v) => ({ emailVerification: v }),
  },
  {
    key: 'forgot-password',
    label: 'Password recovery',
    description:
      'A "Forgot password" link appears on the sign-in page and sends a reset message to the address on the account.',
    read: (d) => d.forgotPassword,
    patch: (v) => ({ forgotPassword: v }),
  },
  {
    key: 'remember-me',
    label: 'Stay signed in',
    description:
      'A "Remember me" checkbox appears on the sign-in page and keeps the session across browser restarts.',
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
            Sign-in methods unavailable
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            This realm carries no settings the console can read.
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
          <h1 className={tokens.header.title}>Sign-in methods</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            What the people of this realm can use to prove who they are.
          </p>
        </div>
      </div>

      <div className={tokens.page.blockGap}>
        {emailDependent && !smtpConfigured && (
          <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
            <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
            <p className='text-xs text-neutral-700 dark:text-neutral-300'>
              Magic link, email verification and password recovery all send a message, and this
              realm has no SMTP configuration recorded. Every one of them fails silently for the
              person signing in until a server is configured under Branding, in the SMTP tab of
              Email templates.
            </p>
          </div>
        )}

        <Section
          title='Identifiers'
          description='What the person types on the sign-in page to name their account.'
        >
          <FieldRow
            label='Accepted identifiers'
            description='Both may be accepted at once. The order decides which one is tried first when a value matches two accounts.'
          >
            <OrderedChoiceCards
              label='Accepted identifiers'
              value={value.loginAliases}
              onChange={(next) => onChange({ loginAliases: next })}
              options={ALIAS_CHOICES}
              minSelectedReason='Select at least one identifier'
            />
            {errors.loginAliases && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.loginAliases}</p>
            )}
          </FieldRow>
        </Section>

        <Section
          title='Passwordless methods'
          description='Offered next to the password field. A password always remains accepted.'
        >
          <CapabilityRows capabilities={METHODS} value={value} onChange={onChange} />

          {value.magicLink && (
            <FieldRow
              label='Magic link validity'
              description='How long the link stays usable after the message is sent. Past it the person has to ask for another one.'
              htmlFor='sign-in-magic-link-ttl'
            >
              <NumberField
                id='sign-in-magic-link-ttl'
                value={value.magicLinkTtl}
                unit='minutes'
                min={1}
                error={errors.magicLinkTtl}
                onChange={(v) => onChange({ magicLinkTtl: v })}
              />
            </FieldRow>
          )}
        </Section>

        <Section
          title='Multi-factor authentication'
          description='A second factor asked once the first one succeeded.'
        >
          <FieldRow
            label='Require a second factor'
            description='Everyone of this realm is asked for a one-time code after their password. Accounts with no second factor enrolled are sent through the enrolment screen before they can continue.'
          >
            <SwitchField
              checked={value.requireMfa}
              onCheckedChange={(v) => onChange({ requireMfa: v })}
            />
          </FieldRow>
        </Section>

        <Section
          title='Account access'
          description='The links and checkboxes the sign-in page offers around the form.'
        >
          <CapabilityRows capabilities={ACCESS} value={value} onChange={onChange} />
        </Section>

        <Section
          title='Sign-in protection'
          description='What happens after repeated failures on the same account.'
        >
          <FieldRow
            label='Failed attempts before lockout'
            description='How many consecutive wrong passwords lock the account. 0 never locks it.'
            htmlFor='sign-in-lockout-threshold'
          >
            <NumberField
              id='sign-in-lockout-threshold'
              value={value.lockoutThreshold}
              unit='attempts'
              min={0}
              error={errors.lockoutThreshold}
              onChange={(v) => onChange({ lockoutThreshold: v })}
            />
          </FieldRow>

          <FieldRow
            label='Lockout duration'
            description='How long a locked account refuses every password before it accepts one again.'
            htmlFor='sign-in-lockout-duration'
          >
            <NumberField
              id='sign-in-lockout-duration'
              value={value.lockoutDuration}
              unit='seconds'
              min={0}
              error={errors.lockoutDuration}
              onChange={(v) => onChange({ lockoutDuration: v })}
            />
          </FieldRow>
        </Section>

        <div className='flex flex-wrap gap-x-6 gap-y-1 text-xs text-neutral-400 dark:text-neutral-500'>
          <span className='inline-flex items-center gap-1.5'>
            <Fingerprint className='size-3.5' strokeWidth={1.75} />
            Passkey {value.passkey ? 'offered' : 'hidden'}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <Link2 className='size-3.5' strokeWidth={1.75} />
            Magic link {value.magicLink ? 'offered' : 'hidden'}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <ShieldCheck className='size-3.5' strokeWidth={1.75} />
            Second factor {value.requireMfa ? 'required' : 'optional'}
          </span>
          <span className='inline-flex items-center gap-1.5'>
            <KeyRound className='size-3.5' strokeWidth={1.75} />
            {value.lockoutThreshold > 0
              ? `Locks after ${value.lockoutThreshold} failed attempts`
              : 'Never locks an account'}
          </span>
        </div>
      </div>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='These changes take effect on the next sign-in of this realm.'
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
    </PageShell>
  )
}
