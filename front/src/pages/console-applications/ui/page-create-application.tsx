import { ApplicationType } from '@/routes/sub-router/applications.router'
import { ArrowLeft, Loader2, Plus, Sparkles, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import { APPLICATION_TONE, getApplicationTypeMeta } from '../types'

export interface CreateApplicationValues {
  name: string
  clientId: string
  callbackUrls: string[]
  allowedOrigins: string[]
}

interface Props {
  type: ApplicationType
  onCancel: () => void
  onBack: () => void
  onSubmit: (values: CreateApplicationValues) => void
  isSubmitting: boolean
}

const slugify = (s: string) =>
  s
    .toLowerCase()
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9-_]+/g, '-')
    .replace(/^-+|-+$/g, '')

interface FieldsConfig {
  showCallbacks: boolean
  callbacksLabel: string
  callbacksHint: string
  callbacksPlaceholder: string
  showOrigins: boolean
  originsLabel?: string
  originsHint?: string
  originsPlaceholder?: string
  callbackRequired: boolean
}

const FIELDS_BY_TYPE: Record<ApplicationType, FieldsConfig> = {
  native: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint:
      'Custom scheme like com.acme.app://callback for mobile, or http://localhost for desktop dev.',
    callbacksPlaceholder: 'com.acme.app://callback',
    showOrigins: false,
    callbackRequired: true,
  },
  spa: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint: 'Where users land after sign-in (e.g. https://app.acme.com/callback).',
    callbacksPlaceholder: 'https://app.acme.com/callback',
    showOrigins: true,
    originsLabel: 'Allowed web origins',
    originsHint: 'Origins allowed to call FerrisKey from the browser (CORS).',
    originsPlaceholder: 'https://app.acme.com',
    callbackRequired: true,
  },
  web: {
    showCallbacks: true,
    callbacksLabel: 'Allowed callback URLs',
    callbacksHint: 'Server-side endpoint that exchanges the auth code for a token.',
    callbacksPlaceholder: 'https://app.acme.com/auth/callback',
    showOrigins: false,
    callbackRequired: true,
  },
  m2m: {
    showCallbacks: false,
    callbacksLabel: '',
    callbacksHint: '',
    callbacksPlaceholder: '',
    showOrigins: false,
    callbackRequired: false,
  },
}

const isValidUrl = (s: string) => {
  if (!s) return false
  // Accept any scheme://host form (incl. custom mobile schemes).
  return /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/.test(s.trim())
}

export default function PageCreateApplication({ type, onCancel, onBack, onSubmit, isSubmitting }: Props) {
  const meta = getApplicationTypeMeta(type)
  const tone = APPLICATION_TONE[meta.tone]
  const fields = FIELDS_BY_TYPE[type]

  const [name, setName] = useState('')
  const [clientIdOverride, setClientIdOverride] = useState<string | null>(null)
  const [callbacks, setCallbacks] = useState<string[]>([''])
  const [origins, setOrigins] = useState<string[]>([''])

  const clientId = clientIdOverride ?? slugify(name)
  const clientIdValid = /^[a-z0-9-_]+$/.test(clientId)

  const callbackUrls = useMemo(
    () => callbacks.map((s) => s.trim()).filter(Boolean),
    [callbacks],
  )
  const originUrls = useMemo(() => origins.map((s) => s.trim()).filter(Boolean), [origins])

  const callbacksValid = !fields.showCallbacks || callbackUrls.every(isValidUrl)
  const originsValid = !fields.showOrigins || originUrls.every(isValidUrl)
  const callbackRequirementMet = !fields.callbackRequired || callbackUrls.length > 0

  const canSubmit =
    name.trim().length > 0 &&
    clientId.trim().length > 0 &&
    clientIdValid &&
    callbacksValid &&
    originsValid &&
    callbackRequirementMet &&
    !isSubmitting

  const addAt = (list: string[], setter: (v: string[]) => void) => setter([...list, ''])
  const removeAt = (list: string[], setter: (v: string[]) => void, idx: number) => {
    const next = list.filter((_, i) => i !== idx)
    setter(next.length ? next : [''])
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!canSubmit) return
    onSubmit({
      name: name.trim(),
      clientId: clientId.trim(),
      callbackUrls,
      allowedOrigins: originUrls,
    })
  }

  return (
    <form onSubmit={handleSubmit} className='flex flex-col gap-8 p-8 md:p-12 max-w-3xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onCancel}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to applications
        </button>
        <div className='flex items-start gap-4'>
          <div className={`h-12 w-12 rounded-md flex items-center justify-center ${tone.bg}`}>
            <meta.icon className={`h-5 w-5 ${tone.fg}`} />
          </div>
          <div className='flex-1'>
            <div className='flex items-center gap-2'>
              <h1 className='text-2xl font-medium tracking-tight'>Configure your {meta.label.toLowerCase()}</h1>
              <span className={`inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                {meta.short}
              </span>
            </div>
            <p className='text-sm text-muted-foreground mt-1'>
              {meta.description} Auth flow: <span className='font-medium text-foreground'>{meta.flow}</span>.
            </p>
          </div>
        </div>
      </div>

      {/* Step indicator */}
      <div className='flex items-center gap-2 text-xs text-muted-foreground'>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 rounded-md bg-muted px-2 py-0.5 hover:bg-muted/70 transition-colors'
        >
          1 · Type
        </button>
        <span className='text-muted-foreground/40'>·</span>
        <span className='inline-flex items-center gap-1.5 rounded-md bg-primary/10 px-2 py-0.5 text-primary font-medium'>
          <Sparkles className='h-3 w-3' />
          Step 2 of 2 · Configure
        </span>
      </div>

      {/* Identity */}
      <div className='flex flex-col gap-5'>
        <Field label='Application name' required hint='How users see this application during sign-in.'>
          <input
            type='text'
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder='Acme Mobile'
            autoFocus
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>

        <Field
          label='Client ID'
          required
          hint='Identifier used in OAuth requests. Auto-derived from the name.'
          error={!clientIdValid && clientId.length > 0 ? 'Only lowercase letters, numbers, hyphens and underscores.' : undefined}
        >
          <input
            type='text'
            value={clientId}
            onChange={(e) => setClientIdOverride(e.target.value)}
            placeholder='acme-mobile'
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>
      </div>

      {/* URLs */}
      {fields.showCallbacks && (
        <UrlListField
          label={fields.callbacksLabel}
          hint={fields.callbacksHint}
          required={fields.callbackRequired}
          placeholder={fields.callbacksPlaceholder}
          values={callbacks}
          onChange={setCallbacks}
          onAdd={() => addAt(callbacks, setCallbacks)}
          onRemove={(i) => removeAt(callbacks, setCallbacks, i)}
          validate={isValidUrl}
        />
      )}

      {fields.showOrigins && (
        <UrlListField
          label={fields.originsLabel ?? ''}
          hint={fields.originsHint}
          required={false}
          placeholder={fields.originsPlaceholder ?? ''}
          values={origins}
          onChange={setOrigins}
          onAdd={() => addAt(origins, setOrigins)}
          onRemove={(i) => removeAt(origins, setOrigins, i)}
          validate={isValidUrl}
        />
      )}

      {/* M2M info banner */}
      {type === 'm2m' && (
        <div className='rounded-md border border-violet-500/30 bg-violet-500/5 p-4 text-sm'>
          <p className='font-medium'>No callback URLs needed</p>
          <p className='text-xs text-muted-foreground mt-1'>
            Machine-to-machine apps authenticate directly with FerrisKey using the
            <code className='font-mono mx-1 px-1 py-0.5 bg-background rounded text-[11px] border border-border'>client_credentials</code>
            grant. We&apos;ll generate a secret you can copy from the application detail page.
          </p>
        </div>
      )}

      {/* Actions */}
      <div className='flex items-center justify-between gap-3 pt-4 border-t border-border'>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors'
        >
          <ArrowLeft className='h-3 w-3' />
          Change type
        </button>
        <div className='flex items-center gap-2'>
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
            className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
          >
            {isSubmitting && <Loader2 className='h-3.5 w-3.5 animate-spin' />}
            {isSubmitting ? 'Creating…' : 'Create application'}
          </button>
        </div>
      </div>
    </form>
  )
}

interface UrlListFieldProps {
  label: string
  hint?: string
  required: boolean
  placeholder: string
  values: string[]
  onChange: (next: string[]) => void
  onAdd: () => void
  onRemove: (index: number) => void
  validate: (s: string) => boolean
}

function UrlListField({ label, hint, required, placeholder, values, onChange, onAdd, onRemove, validate }: UrlListFieldProps) {
  return (
    <div className='flex flex-col gap-2'>
      <div className='flex items-center gap-2'>
        <span className='text-sm font-medium'>{label}</span>
        {required ? (
          <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>required</span>
        ) : (
          <span className='text-[10px] uppercase tracking-wider text-muted-foreground'>optional</span>
        )}
      </div>
      <div className='flex flex-col gap-2'>
        {values.map((v, i) => {
          const invalid = v.length > 0 && !validate(v)
          return (
            <div key={i} className='flex items-center gap-2'>
              <input
                type='text'
                value={v}
                onChange={(e) => {
                  const next = [...values]
                  next[i] = e.target.value
                  onChange(next)
                }}
                placeholder={placeholder}
                className={`flex-1 rounded-md border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:ring-1 focus:ring-primary/30 ${
                  invalid ? 'border-red-500/40 focus:border-red-500/60' : 'border-border focus:border-primary/40'
                }`}
              />
              <button
                type='button'
                onClick={() => onRemove(i)}
                className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-30'
                disabled={values.length === 1 && !v}
                aria-label='Remove URL'
              >
                <X className='h-3.5 w-3.5' />
              </button>
            </div>
          )
        })}
        <button
          type='button'
          onClick={onAdd}
          className='inline-flex items-center gap-1.5 rounded-md border border-dashed border-border bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:border-primary/40 transition-colors self-start'
        >
          <Plus className='h-3 w-3' />
          Add another URL
        </button>
      </div>
      {hint && <p className='text-xs text-muted-foreground'>{hint}</p>}
    </div>
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
