import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { isServiceAccount } from '@/utils'
import {
  AlertTriangle,
  ArrowLeft,
  AtSign,
  BadgeCheck,
  Fingerprint,
  KeyRound,
  LifeBuoy,
  Link2,
  Loader2,
  Lock,
  Mail,
  Plus,
  Power,
  ShieldUser,
  Trash2,
  UserCog,
  X,
} from 'lucide-react'
import { useMemo, useState } from 'react'

import CredentialOverview = Schemas.CredentialOverview
import RequiredAction = Schemas.RequiredAction
import Role = Schemas.Role
import User = Schemas.User

export interface IdentityProfileValues {
  firstname: string
  lastname: string
  email: string
  emailVerified: boolean
  enabled: boolean
  requiredActions: RequiredAction[]
}

const REQUIRED_ACTION_CATALOG: {
  key: RequiredAction
  label: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  tone: 'blue' | 'amber' | 'emerald' | 'violet'
}[] = [
  {
    key: 'configure_passkey',
    label: 'Enroll a passkey',
    description: 'User must register a WebAuthn passkey at next sign-in.',
    icon: Fingerprint,
    tone: 'blue',
  },
  {
    key: 'configure_otp',
    label: 'Set up TOTP',
    description: 'User must scan a TOTP code with their authenticator app.',
    icon: KeyRound,
    tone: 'violet',
  },
  {
    key: 'update_password',
    label: 'Reset password',
    description: 'User must choose a new password at next sign-in.',
    icon: ShieldUser,
    tone: 'amber',
  },
  {
    key: 'verify_email',
    label: 'Verify email',
    description: 'User must click a verification link sent by email.',
    icon: Mail,
    tone: 'emerald',
  },
]

const ACTION_TONE: Record<'blue' | 'amber' | 'emerald' | 'violet', { bg: string; fg: string; border: string }> = {
  blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500', border: 'border-blue-500/30' },
  amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500', border: 'border-amber-500/30' },
  emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500', border: 'border-emerald-500/30' },
  violet: { bg: 'bg-violet-500/10', fg: 'text-violet-500', border: 'border-violet-500/30' },
}

interface Props {
  identity: User | null
  credentials: CredentialOverview[]
  roles: Role[]
  isLoading: boolean
  isUpdating: boolean
  isDeleting: boolean
  onBack: () => void
  onSave: (values: IdentityProfileValues) => void
  onDelete: () => void
}

const initials = (u: User) => {
  const f = (u.firstname ?? '').trim()
  const l = (u.lastname ?? '').trim()
  if (f || l) return `${f[0] ?? ''}${l[0] ?? ''}`.toUpperCase() || u.username[0]?.toUpperCase()
  return u.username.slice(0, 2).toUpperCase()
}

const displayName = (u: User) => {
  if (isServiceAccount(u)) return 'Service account'
  const full = [u.firstname, u.lastname].filter(Boolean).join(' ')
  return full || u.username
}

const formatDate = (iso: string) => {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' })
}

interface CredentialMeta {
  label: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue' | 'violet' | 'amber' | 'muted'
}

const CREDENTIAL_TONE: Record<CredentialMeta['tone'], { bg: string; fg: string; ring: string }> = {
  emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500', ring: 'ring-emerald-500/30' },
  blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500', ring: 'ring-blue-500/30' },
  violet: { bg: 'bg-violet-500/10', fg: 'text-violet-500', ring: 'ring-violet-500/30' },
  amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500', ring: 'ring-amber-500/30' },
  muted: { bg: 'bg-muted', fg: 'text-muted-foreground', ring: 'ring-border' },
}

function describeCredential(type: string): CredentialMeta {
  const t = type.toLowerCase()
  if (t.includes('webauthn') || t.includes('passkey')) {
    return {
      label: 'Passkey',
      description: 'Passwordless · WebAuthn',
      icon: Fingerprint,
      tone: 'blue',
    }
  }
  if (t.includes('otp') || t.includes('totp')) {
    return {
      label: 'Authenticator app',
      description: 'TOTP · 6-digit code',
      icon: KeyRound,
      tone: 'violet',
    }
  }
  if (t.includes('magic')) {
    return {
      label: 'Magic link',
      description: 'Email-based sign-in',
      icon: Mail,
      tone: 'emerald',
    }
  }
  if (t.includes('recovery')) {
    return {
      label: 'Recovery codes',
      description: 'One-time backup codes',
      icon: LifeBuoy,
      tone: 'amber',
    }
  }
  if (t.includes('federated')) {
    return {
      label: 'Federated identity',
      description: 'Linked external provider',
      icon: Link2,
      tone: 'blue',
    }
  }
  if (t.includes('password')) {
    return {
      label: 'Password',
      description: 'Long-lived secret',
      icon: Lock,
      tone: 'amber',
    }
  }
  return {
    label: type || 'Unknown method',
    description: 'Custom credential',
    icon: KeyRound,
    tone: 'muted',
  }
}

const isValidEmail = (s: string) => !s || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s)

export default function PageIdentityDetail(props: Props) {
  const { identity, isLoading } = props
  if (isLoading || !identity) {
    return (
      <div className='flex flex-col gap-6 p-8 md:p-12'>
        <Skeleton className='h-4 w-32' />
        <div className='flex items-center gap-4'>
          <Skeleton className='h-16 w-16 rounded-md' />
          <div className='space-y-2'>
            <Skeleton className='h-6 w-48' />
            <Skeleton className='h-3 w-32' />
          </div>
        </div>
        <Skeleton className='h-64 w-full rounded-md' />
      </div>
    )
  }
  // Re-mount inner state when the underlying identity changes, so the form
  // initializes from fresh props without effect-based syncing.
  return <IdentityDetailLoaded key={identity.id} {...props} identity={identity} />
}

interface LoadedProps extends Omit<Props, 'identity'> {
  identity: User
}

function IdentityDetailLoaded({
  identity,
  credentials,
  roles,
  isUpdating,
  isDeleting,
  onBack,
  onSave,
  onDelete,
}: LoadedProps) {
  const [firstname, setFirstname] = useState(identity.firstname ?? '')
  const [lastname, setLastname] = useState(identity.lastname ?? '')
  const [email, setEmail] = useState(identity.email ?? '')
  const [emailVerified, setEmailVerified] = useState(identity.email_verified)
  const [enabled, setEnabled] = useState(identity.enabled)
  const [requiredActions, setRequiredActions] = useState<RequiredAction[]>(identity.required_actions ?? [])
  const [confirmingDelete, setConfirmingDelete] = useState(false)

  const initialActions = identity.required_actions ?? []
  const actionsChanged =
    requiredActions.length !== initialActions.length ||
    requiredActions.some((a) => !initialActions.includes(a))

  const dirty = useMemo(
    () =>
      firstname !== (identity.firstname ?? '') ||
      lastname !== (identity.lastname ?? '') ||
      email !== (identity.email ?? '') ||
      emailVerified !== identity.email_verified ||
      enabled !== identity.enabled ||
      actionsChanged,
    [identity, firstname, lastname, email, emailVerified, enabled, actionsChanged],
  )

  const emailValid = isValidEmail(email)
  const canSave = dirty && emailValid && !isUpdating

  const toggleRequiredAction = (action: RequiredAction) => {
    setRequiredActions((prev) =>
      prev.includes(action) ? prev.filter((a) => a !== action) : [...prev, action],
    )
  }

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSave) return
    onSave({
      firstname: firstname.trim(),
      lastname: lastname.trim(),
      email: email.trim(),
      emailVerified,
      enabled,
      requiredActions,
    })
  }

  return (
    <div className='flex flex-col gap-8 p-8 md:p-12 max-w-4xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to identities
        </button>
        <div className='flex items-start justify-between gap-4 flex-wrap'>
          <div className='flex items-center gap-4'>
            <div className='h-16 w-16 rounded-md bg-primary/10 text-primary flex items-center justify-center text-lg font-semibold'>
              {initials(identity)}
            </div>
            <div>
              <h1 className='text-2xl font-medium tracking-tight flex items-center gap-3'>
                {displayName(identity)}
                {!enabled && (
                  <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                    Disabled
                  </span>
                )}
                {emailVerified && enabled && (
                  <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/30 uppercase tracking-wide'>
                    <BadgeCheck className='h-3 w-3' />
                    Verified
                  </span>
                )}
              </h1>
              <p className='text-sm text-muted-foreground mt-1 font-mono'>@{identity.username}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Required actions banner — read-only summary of currently persisted actions */}
      {(identity.required_actions?.length ?? 0) > 0 && (
        <div className='flex items-start gap-3 rounded-md border border-amber-500/40 bg-amber-50 dark:bg-amber-500/10 p-4'>
          <UserCog className='h-4 w-4 text-amber-600 dark:text-amber-400 mt-0.5 shrink-0' />
          <div className='flex-1'>
            <p className='text-sm font-medium text-foreground'>
              {identity.required_actions.length} action{identity.required_actions.length > 1 ? 's' : ''} pending at next sign-in
            </p>
            <p className='text-xs text-muted-foreground mt-0.5'>
              Manage them below in the &quot;Require at next sign-in&quot; section.
            </p>
          </div>
        </div>
      )}

      {/* Profile */}
      <Section title='Profile' description='Identity attributes the customer can also edit from their account settings.'>
        <form onSubmit={handleSave} className='flex flex-col gap-5'>
          <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
            <Field label='First name' optional>
              <input
                type='text'
                value={firstname}
                onChange={(e) => setFirstname(e.target.value)}
                placeholder='Ada'
                className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
            <Field label='Last name' optional>
              <input
                type='text'
                value={lastname}
                onChange={(e) => setLastname(e.target.value)}
                placeholder='Lovelace'
                className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </Field>
          </div>

          <Field label='Email' optional error={!emailValid ? 'Enter a valid email address.' : undefined}>
            <div className='relative'>
              <AtSign className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
              <input
                type='email'
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder='ada@acme.com'
                className='w-full rounded-md border border-border bg-background pl-9 pr-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
            </div>
          </Field>

          <div className='flex flex-col gap-2'>
            <Toggle
              label='Email verified'
              description='Treat the email as confirmed. Customers with verified email can use magic links and password reset.'
              icon={BadgeCheck}
              tone='emerald'
              checked={emailVerified}
              onChange={setEmailVerified}
            />
            <Toggle
              label='Account enabled'
              description='Disabled accounts cannot sign in. Existing sessions remain until they expire.'
              icon={Power}
              tone='blue'
              checked={enabled}
              onChange={setEnabled}
            />
          </div>

          <div className='flex items-center justify-end gap-2 pt-2 border-t border-border'>
            <button
              type='button'
              onClick={() => {
                setFirstname(identity.firstname ?? '')
                setLastname(identity.lastname ?? '')
                setEmail(identity.email ?? '')
                setEmailVerified(identity.email_verified)
                setEnabled(identity.enabled)
                setRequiredActions(identity.required_actions ?? [])
              }}
              disabled={!dirty || isUpdating}
              className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors disabled:opacity-40 disabled:cursor-not-allowed'
            >
              Reset
            </button>
            <button
              type='submit'
              disabled={!canSave}
              className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
            >
              {isUpdating && <Loader2 className='h-3.5 w-3.5 animate-spin' />}
              {isUpdating ? 'Saving…' : 'Save changes'}
            </button>
          </div>
        </form>
      </Section>

      {/* Sign-in methods */}
      <Section title='Sign-in methods' description='Credentials this identity can use to authenticate.'>
        {credentials.length === 0 ? (
          <p className='text-sm text-muted-foreground rounded-md border border-dashed border-border bg-muted/20 p-6 text-center'>
            No credentials enrolled yet.
          </p>
        ) : (
          <div className='grid grid-cols-1 sm:grid-cols-2 gap-3'>
            {credentials.map((c) => {
              const meta = describeCredential(c.credential_type)
              const tone = CREDENTIAL_TONE[meta.tone]
              return (
                <div
                  key={c.id}
                  className={`flex items-start gap-3 rounded-md border bg-card/40 p-3 ${tone.ring.replace('ring-', 'border-')}`}
                >
                  <div className={`h-10 w-10 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}>
                    <meta.icon className={`h-4 w-4 ${tone.fg}`} />
                  </div>
                  <div className='flex-1 min-w-0'>
                    <div className='flex items-center justify-between gap-2'>
                      <span className='text-sm font-medium truncate'>{meta.label}</span>
                      <span
                        className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide shrink-0 ${tone.bg} ${tone.fg}`}
                      >
                        Active
                      </span>
                    </div>
                    <p className='text-xs text-muted-foreground'>{meta.description}</p>
                    {c.user_label && (
                      <p className='text-xs text-foreground mt-1 truncate' title={c.user_label}>
                        {c.user_label}
                      </p>
                    )}
                    <div className='flex items-center gap-3 mt-2 text-[11px] text-muted-foreground'>
                      <span>Added {formatDate(c.created_at)}</span>
                      <span className='text-muted-foreground/60'>·</span>
                      <code className='font-mono text-[10px] truncate'>{c.credential_type}</code>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </Section>

      {/* Require at next sign-in */}
      <Section
        title='Require at next sign-in'
        description='Force the customer to enroll a factor or take a step the next time they sign in. Saved with the rest of the form.'
      >
        <div className='grid grid-cols-1 sm:grid-cols-2 gap-3'>
          {REQUIRED_ACTION_CATALOG.map((action) => {
            const active = requiredActions.includes(action.key)
            const tone = ACTION_TONE[action.tone]
            return (
              <button
                key={action.key}
                type='button'
                onClick={() => toggleRequiredAction(action.key)}
                className={`group flex items-start gap-3 rounded-md border p-4 text-left transition ${
                  active
                    ? `${tone.border} ${tone.bg}`
                    : 'border-border bg-card/40 hover:border-primary/30 hover:bg-muted/40'
                }`}
              >
                <div className={`h-9 w-9 rounded-md flex items-center justify-center ${tone.bg}`}>
                  <action.icon className={`h-4 w-4 ${tone.fg}`} />
                </div>
                <div className='flex-1 min-w-0'>
                  <div className='flex items-center gap-2'>
                    <p className='text-sm font-medium'>{action.label}</p>
                    {active && (
                      <span className={`inline-flex items-center gap-0.5 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                        Required
                      </span>
                    )}
                  </div>
                  <p className='text-xs text-muted-foreground mt-1'>{action.description}</p>
                  <p className='text-[10px] font-mono text-muted-foreground/70 mt-1'>{action.key}</p>
                </div>
                <div className='shrink-0'>
                  {active ? (
                    <X className={`h-4 w-4 ${tone.fg}`} />
                  ) : (
                    <Plus className='h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors' />
                  )}
                </div>
              </button>
            )
          })}
        </div>
        {actionsChanged && (
          <p className='text-xs text-amber-600 inline-flex items-center gap-1.5'>
            <AlertTriangle className='h-3 w-3' />
            Pending changes. Click "Save changes" above to apply.
          </p>
        )}
      </Section>

      {/* Roles */}
      <Section title='Roles' description='Permissions granted to this identity through its roles.'>
        {roles.length === 0 ? (
          <p className='text-sm text-muted-foreground rounded-md border border-dashed border-border bg-muted/20 p-6 text-center'>
            No roles assigned.
          </p>
        ) : (
          <div className='flex flex-wrap gap-2'>
            {roles.map((r) => (
              <span
                key={r.id}
                className='inline-flex items-center gap-1.5 rounded-md border border-border bg-card/40 px-2.5 py-1 text-sm'
              >
                <ShieldUser className='h-3.5 w-3.5 text-primary' />
                <span className='font-medium'>{r.name}</span>
                {(r.permissions?.length ?? 0) > 0 && (
                  <span className='text-xs text-muted-foreground'>· {r.permissions?.length}</span>
                )}
              </span>
            ))}
          </div>
        )}
      </Section>

      {/* Account info */}
      <Section title='Account info' description='Read-only metadata about this identity.'>
        <dl className='grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm'>
          <InfoRow label='Identity ID' value={<code className='font-mono text-xs'>{identity.id}</code>} />
          <InfoRow label='Username' value={<code className='font-mono text-xs'>{identity.username}</code>} />
          <InfoRow label='Created' value={formatDate(identity.created_at)} />
          <InfoRow label='Last update' value={formatDate(identity.updated_at)} />
        </dl>
      </Section>

      {/* Danger zone */}
      <Section
        title='Danger zone'
        description='Destructive actions. These cannot be undone.'
        tone='destructive'
      >
        <div className='flex items-center justify-between gap-4 rounded-md border border-red-500/30 bg-red-500/5 p-4'>
          <div className='flex items-start gap-3 min-w-0'>
            <AlertTriangle className='h-4 w-4 text-red-500 mt-0.5 shrink-0' />
            <div>
              <p className='text-sm font-medium'>Delete this identity</p>
              <p className='text-xs text-muted-foreground mt-0.5'>
                Removes the account, its credentials and audit traces. Active sessions are revoked.
              </p>
            </div>
          </div>
          {confirmingDelete ? (
            <div className='flex items-center gap-2'>
              <button
                type='button'
                onClick={() => setConfirmingDelete(false)}
                className='rounded-md border border-border bg-background px-3 py-1.5 text-xs font-medium hover:bg-muted transition-colors'
              >
                Cancel
              </button>
              <button
                type='button'
                onClick={onDelete}
                disabled={isDeleting}
                className='inline-flex items-center gap-1.5 rounded-md bg-red-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-red-600 transition-colors disabled:opacity-50'
              >
                {isDeleting && <Loader2 className='h-3 w-3 animate-spin' />}
                {isDeleting ? 'Deleting…' : 'Confirm delete'}
              </button>
            </div>
          ) : (
            <button
              type='button'
              onClick={() => setConfirmingDelete(true)}
              className='inline-flex items-center gap-1.5 rounded-md border border-red-500/40 bg-background px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-500/10 transition-colors'
            >
              <Trash2 className='h-3 w-3' />
              Delete identity
            </button>
          )}
        </div>
      </Section>
    </div>
  )
}

interface SectionProps {
  title: string
  description?: string
  tone?: 'default' | 'destructive'
  children: React.ReactNode
}

function Section({ title, description, tone = 'default', children }: SectionProps) {
  return (
    <section className='flex flex-col gap-4'>
      <div>
        <h2
          className={`text-sm font-semibold ${tone === 'destructive' ? 'text-red-600' : 'text-foreground'}`}
        >
          {title}
        </h2>
        {description && <p className='text-xs text-muted-foreground mt-1'>{description}</p>}
      </div>
      {children}
    </section>
  )
}

interface FieldProps {
  label: string
  hint?: string
  required?: boolean
  optional?: boolean
  error?: string
  children: React.ReactNode
}

function Field({ label, hint, required, optional, error, children }: FieldProps) {
  return (
    <label className='flex flex-col gap-1.5'>
      <div className='flex items-center gap-2'>
        <span className='text-sm font-medium'>{label}</span>
        {required && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>required</span>}
        {optional && <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>optional</span>}
      </div>
      {children}
      {error ? (
        <p className='text-xs text-red-500'>{error}</p>
      ) : (
        hint && <p className='text-xs text-muted-foreground'>{hint}</p>
      )}
    </label>
  )
}

interface ToggleProps {
  label: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue'
  checked: boolean
  onChange: (v: boolean) => void
}

function Toggle({ label, description, icon: Icon, tone, checked, onChange }: ToggleProps) {
  const toneFg = tone === 'emerald' ? 'text-emerald-500' : 'text-blue-500'
  return (
    <label className='flex items-start gap-3 rounded-md border border-border bg-card/40 p-3 cursor-pointer hover:bg-muted/40 transition-colors'>
      <input
        type='checkbox'
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className='mt-0.5 h-4 w-4 rounded border-border accent-primary'
      />
      <div className='flex-1'>
        <div className='flex items-center gap-2'>
          <Icon className={`h-4 w-4 ${toneFg}`} />
          <span className='text-sm font-medium'>{label}</span>
        </div>
        <p className='text-xs text-muted-foreground mt-1'>{description}</p>
      </div>
    </label>
  )
}

function InfoRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className='flex flex-col gap-0.5'>
      <dt className='text-xs text-muted-foreground'>{label}</dt>
      <dd className='text-sm'>{value}</dd>
    </div>
  )
}
