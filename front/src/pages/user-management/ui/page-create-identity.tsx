import { ArrowLeft, AtSign, BadgeCheck, UserPlus } from 'lucide-react'
import { useMemo, useState } from 'react'

export interface CreateIdentityValues {
  username: string
  email: string
  firstname: string
  lastname: string
  emailVerified: boolean
}

interface Props {
  onCancel: () => void
  onSubmit: (values: CreateIdentityValues) => void
  isSubmitting: boolean
}

const slugifyUsername = (s: string) =>
  s
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9._-]+/g, '.')
    .replace(/^\.+|\.+$/g, '')

const isValidEmail = (s: string) => !s || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s)

export default function PageCreateIdentity({ onCancel, onSubmit, isSubmitting }: Props) {
  const [firstname, setFirstname] = useState('')
  const [lastname, setLastname] = useState('')
  const [email, setEmail] = useState('')
  const [usernameOverride, setUsernameOverride] = useState<string | null>(null)
  const [emailVerified, setEmailVerified] = useState(false)

  const derivedUsername = useMemo(() => {
    if (email) {
      const local = email.split('@')[0]
      if (local) return slugifyUsername(local)
    }
    const full = `${firstname} ${lastname}`.trim()
    if (full) return slugifyUsername(full.replace(/\s+/g, '.'))
    return ''
  }, [email, firstname, lastname])

  const username = usernameOverride ?? derivedUsername
  const emailValid = isValidEmail(email)
  const canSubmit =
    username.trim().length > 0 && emailValid && !isSubmitting

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    onSubmit({
      username: username.trim(),
      email: email.trim(),
      firstname: firstname.trim(),
      lastname: lastname.trim(),
      emailVerified,
    })
  }

  return (
    <form onSubmit={handleSubmit} className='flex flex-col gap-8 p-8 md:p-12 max-w-2xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onCancel}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to identities
        </button>
        <div className='flex items-start gap-4'>
          <div className='h-12 w-12 rounded-md bg-primary/10 flex items-center justify-center'>
            <UserPlus className='h-5 w-5 text-primary' />
          </div>
          <div>
            <h1 className='text-2xl font-medium tracking-tight'>Create identity</h1>
            <p className='text-sm text-muted-foreground mt-1'>
              Add a customer account. They&apos;ll be able to sign in once a credential is set or an
              invite is sent.
            </p>
          </div>
        </div>
      </div>

      {/* Fields */}
      <div className='flex flex-col gap-5'>
        <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
          <Field label='First name' optional>
            <input
              type='text'
              value={firstname}
              onChange={(e) => setFirstname(e.target.value)}
              placeholder='Ada'
              autoFocus
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

        <Field
          label='Email'
          optional
          hint='Used for verification, magic links and password resets.'
          error={!emailValid ? 'Enter a valid email address.' : undefined}
        >
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

        <Field
          label='Username'
          required
          hint='How the identity is identified internally. Auto-derived from email or name.'
        >
          <input
            type='text'
            value={username}
            onChange={(e) => setUsernameOverride(e.target.value)}
            placeholder='ada.lovelace'
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <label className='flex items-start gap-3 rounded-md border border-border bg-card/40 p-3 cursor-pointer hover:bg-muted/40 transition-colors'>
          <input
            type='checkbox'
            checked={emailVerified}
            onChange={(e) => setEmailVerified(e.target.checked)}
            className='mt-0.5 h-4 w-4 rounded border-border accent-primary'
          />
          <div className='flex-1'>
            <div className='flex items-center gap-2'>
              <BadgeCheck className='h-4 w-4 text-emerald-500' />
              <span className='text-sm font-medium'>Mark email as verified</span>
            </div>
            <p className='text-xs text-muted-foreground mt-1'>
              Skip the verification email. Use only when you trust the source of this address.
            </p>
          </div>
        </label>
      </div>

      {/* Actions */}
      <div className='flex items-center justify-end gap-2 pt-2 border-t border-border'>
        <button
          type='button'
          onClick={onCancel}
          className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors'
        >
          Cancel
        </button>
        <button
          type='submit'
          disabled={!canSubmit}
          className='rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
        >
          {isSubmitting ? 'Creating…' : 'Create identity'}
        </button>
      </div>
    </form>
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
