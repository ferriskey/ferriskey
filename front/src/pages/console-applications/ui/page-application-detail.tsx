import { Schemas } from '@/api/api.client'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import { Switch } from '@/components/ui/switch'
import {
  ArrowLeft,
  Check,
  Copy,
  Eye,
  EyeOff,
  Loader2,
  Plus,
  ShieldOff,
  Trash2,
  X,
} from 'lucide-react'
import { useMemo, useState } from 'react'
import { APPLICATION_TONE, getApplicationTypeMeta, inferApplicationType } from '../types'

import Client = Schemas.Client

export interface ApplicationSettingsValues {
  name: string
  enabled: boolean
  directAccessGrantsEnabled: boolean
  oauthDeviceCodeGrantEnabled: boolean
  accessTokenLifetime: number | null
  refreshTokenLifetime: number | null
  idTokenLifetime: number | null
  temporaryTokenLifetime: number | null
}

interface Props {
  client: Client
  isSaving: boolean
  uriPending: boolean
  onBack: () => void
  onSave: (values: ApplicationSettingsValues) => Promise<void>
  onDelete: () => void
  onAddRedirectUri: (value: string) => void
  onDeleteRedirectUri: (redirectUriId: string) => void
}

const isValidUrl = (s: string) => /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/.test(s.trim())

function settingsFromClient(c: Client): ApplicationSettingsValues {
  return {
    name: c.name ?? '',
    enabled: c.enabled,
    directAccessGrantsEnabled: c.direct_access_grants_enabled,
    oauthDeviceCodeGrantEnabled: c.oauth_device_code_grant_enabled,
    accessTokenLifetime: c.access_token_lifetime ?? null,
    refreshTokenLifetime: c.refresh_token_lifetime ?? null,
    idTokenLifetime: c.id_token_lifetime ?? null,
    temporaryTokenLifetime: c.temporary_token_lifetime ?? null,
  }
}

export default function PageApplicationDetail({
  client,
  isSaving,
  uriPending,
  onBack,
  onSave,
  onDelete,
  onAddRedirectUri,
  onDeleteRedirectUri,
}: Props) {
  // The parent remounts this component (key={client.id}) when navigating to a
  // different application, so lazy-initialising from the client is sufficient.
  const [values, setValues] = useState<ApplicationSettingsValues>(() => settingsFromClient(client))
  const [newUri, setNewUri] = useState('')

  const baseline = useMemo(() => settingsFromClient(client), [client])
  const hasChanges = useMemo(
    () => JSON.stringify(values) !== JSON.stringify(baseline),
    [values, baseline],
  )

  const appType = inferApplicationType(client)
  const meta = getApplicationTypeMeta(appType)
  const tone = APPLICATION_TONE[meta.tone]
  const isConfidential = client.client_type === 'confidential'
  // Browserless (device) and machine-to-machine clients don't use redirect URIs.
  const showRedirectUris = appType !== 'm2m' && appType !== 'device'

  const set = <K extends keyof ApplicationSettingsValues>(key: K, value: ApplicationSettingsValues[K]) =>
    setValues((prev) => ({ ...prev, [key]: value }))

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!hasChanges || isSaving) return
    void onSave(values)
  }

  const handleAddUri = () => {
    const v = newUri.trim()
    if (!v || !isValidUrl(v)) return
    onAddRedirectUri(v)
    setNewUri('')
  }

  return (
    <form onSubmit={handleSubmit} className='flex flex-col gap-6 p-8 md:p-12 max-w-3xl'>
      {/* Header */}
      <div>
        <button
          type='button'
          onClick={onBack}
          className='inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4'
        >
          <ArrowLeft className='h-3.5 w-3.5' />
          Back to applications
        </button>
        <div className='flex items-start gap-4'>
          <div className={`h-12 w-12 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}>
            <meta.icon className={`h-5 w-5 ${tone.fg}`} />
          </div>
          <div className='flex-1 min-w-0'>
            <div className='flex items-center gap-2'>
              <h1 className='text-2xl font-medium tracking-tight truncate'>{client.name || client.client_id}</h1>
              <span className={`inline-flex items-center rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                {meta.short}
              </span>
              {!values.enabled && (
                <span className='inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                  <ShieldOff className='h-2.5 w-2.5' />
                  Off
                </span>
              )}
            </div>
            <p className='text-sm text-muted-foreground mt-1'>
              {meta.description} Auth flow: <span className='font-medium text-foreground'>{meta.flow}</span>.
            </p>
          </div>
        </div>
      </div>

      {/* General */}
      <Section title='General' description='Name and availability of this application.'>
        <Field label='Application name' hint='How users see this application during sign-in.'>
          <input
            type='text'
            value={values.name}
            onChange={(e) => set('name', e.target.value)}
            className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
          />
        </Field>
        <ToggleRow
          label='Enabled'
          description='Disabled applications cannot start a sign-in flow.'
          checked={values.enabled}
          onChange={(v) => set('enabled', v)}
        />
      </Section>

      {/* Credentials */}
      <Section title='Credentials' description='Identifiers used in OAuth / OIDC requests.'>
        <CopyField label='Client ID' value={client.client_id} mono />
        {isConfidential && (
          client.secret ? (
            <SecretField label='Client secret' value={client.secret} />
          ) : (
            <p className='text-xs text-muted-foreground'>
              The client secret is hidden. Rotate or reveal it from the admin console.
            </p>
          )
        )}
      </Section>

      {/* Redirect URIs */}
      {showRedirectUris && (
        <Section
          title='Redirect URIs'
          description='Allowed callback URLs FerrisKey may redirect to after sign-in.'
        >
          <div className='flex flex-col gap-2'>
            {(client.redirect_uris ?? []).length === 0 && (
              <p className='text-xs text-muted-foreground'>No redirect URIs registered yet.</p>
            )}
            {(client.redirect_uris ?? []).map((uri) => (
              <div
                key={uri.id}
                className='flex items-center gap-2 rounded-md border border-border bg-background px-3 py-2'
              >
                <span className='flex-1 text-sm font-mono truncate'>{uri.value}</span>
                <button
                  type='button'
                  onClick={() => onDeleteRedirectUri(uri.id)}
                  disabled={uriPending}
                  className='inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:text-red-500 hover:bg-muted transition-colors disabled:opacity-40'
                  aria-label='Remove redirect URI'
                >
                  <X className='h-3.5 w-3.5' />
                </button>
              </div>
            ))}
            <div className='flex items-center gap-2'>
              <input
                type='text'
                value={newUri}
                onChange={(e) => setNewUri(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    handleAddUri()
                  }
                }}
                placeholder='https://app.acme.com/callback'
                className='flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
              />
              <button
                type='button'
                onClick={handleAddUri}
                disabled={uriPending || !newUri.trim() || !isValidUrl(newUri)}
                className='inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-2 text-sm font-medium hover:bg-muted transition-colors disabled:opacity-40 disabled:cursor-not-allowed'
              >
                {uriPending ? <Loader2 className='h-3.5 w-3.5 animate-spin' /> : <Plus className='h-3.5 w-3.5' />}
                Add
              </button>
            </div>
          </div>
        </Section>
      )}

      {/* OAuth grants */}
      <Section title='Grants' description='OAuth grant types this application is allowed to use.'>
        <ToggleRow
          label='Direct access grants'
          description='Allow exchanging a username/password directly for tokens (Resource Owner Password Credentials).'
          checked={values.directAccessGrantsEnabled}
          onChange={(v) => set('directAccessGrantsEnabled', v)}
        />
        <ToggleRow
          label='Device authorization grant'
          description='Allow browserless clients (CLI, IoT) to obtain tokens via a verification code (RFC 8628).'
          checked={values.oauthDeviceCodeGrantEnabled}
          onChange={(v) => set('oauthDeviceCodeGrantEnabled', v)}
        />
      </Section>

      {/* Token lifetimes */}
      <Section
        title='Token lifetimes'
        description='Override realm defaults for this application. Leave empty to inherit the realm setting.'
      >
        <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
          <DurationField
            label='Access token'
            value={values.accessTokenLifetime}
            onChange={(v) => set('accessTokenLifetime', v)}
          />
          <DurationField
            label='Refresh token'
            value={values.refreshTokenLifetime}
            onChange={(v) => set('refreshTokenLifetime', v)}
          />
          <DurationField
            label='ID token'
            value={values.idTokenLifetime}
            onChange={(v) => set('idTokenLifetime', v)}
          />
          <DurationField
            label='Temporary token'
            value={values.temporaryTokenLifetime}
            onChange={(v) => set('temporaryTokenLifetime', v)}
          />
        </div>
      </Section>

      {/* Danger zone */}
      <Section title='Danger zone' description='Irreversible actions.' tone='danger'>
        <div className='flex items-center justify-between gap-4'>
          <div>
            <p className='text-sm font-medium'>Delete this application</p>
            <p className='text-xs text-muted-foreground mt-0.5'>
              Removes the client and revokes its ability to authenticate. This cannot be undone.
            </p>
          </div>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <button
                type='button'
                className='inline-flex items-center gap-1.5 rounded-md border border-red-500/40 bg-background px-3 py-2 text-sm font-medium text-red-500 hover:bg-red-500/10 transition-colors shrink-0'
              >
                <Trash2 className='h-3.5 w-3.5' />
                Delete
              </button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Delete {client.name || client.client_id}?</AlertDialogTitle>
                <AlertDialogDescription>
                  This permanently deletes the application and all of its configuration. Any
                  integration using this client will stop working immediately.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction
                  onClick={onDelete}
                  className='bg-red-500 text-white hover:bg-red-500/90'
                >
                  Delete application
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </Section>

      {/* Save bar */}
      <div className='sticky bottom-0 -mx-8 md:-mx-12 px-8 md:px-12 py-4 border-t border-border bg-background/80 backdrop-blur flex items-center justify-end gap-3'>
        {hasChanges && (
          <button
            type='button'
            onClick={() => setValues(baseline)}
            className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors'
          >
            Discard
          </button>
        )}
        <button
          type='submit'
          disabled={!hasChanges || isSaving}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
        >
          {isSaving && <Loader2 className='h-3.5 w-3.5 animate-spin' />}
          {isSaving ? 'Saving…' : 'Save changes'}
        </button>
      </div>
    </form>
  )
}

interface SectionProps {
  title: string
  description?: string
  tone?: 'default' | 'danger'
  children: React.ReactNode
}

function Section({ title, description, tone = 'default', children }: SectionProps) {
  return (
    <section
      className={`rounded-md border bg-card/40 p-5 flex flex-col gap-4 ${
        tone === 'danger' ? 'border-red-500/30' : 'border-border'
      }`}
    >
      <div>
        <h2 className='text-sm font-semibold'>{title}</h2>
        {description && <p className='text-xs text-muted-foreground mt-0.5'>{description}</p>}
      </div>
      {children}
    </section>
  )
}

interface FieldProps {
  label: string
  hint?: string
  children: React.ReactNode
}

function Field({ label, hint, children }: FieldProps) {
  return (
    <label className='flex flex-col gap-1.5'>
      <span className='text-sm font-medium'>{label}</span>
      {children}
      {hint && <p className='text-xs text-muted-foreground'>{hint}</p>}
    </label>
  )
}

interface ToggleRowProps {
  label: string
  description: string
  checked: boolean
  onChange: (checked: boolean) => void
}

function ToggleRow({ label, description, checked, onChange }: ToggleRowProps) {
  return (
    <div className='flex items-center justify-between gap-5 rounded-md border border-border p-3'>
      <div className='space-y-0.5'>
        <p className='text-sm font-medium'>{label}</p>
        <p className='text-xs text-muted-foreground'>{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onChange} />
    </div>
  )
}

function CopyField({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  const [copied, setCopied] = useState(false)
  const copy = () => {
    void navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }
  return (
    <Field label={label}>
      <div className='flex items-center gap-2'>
        <input
          readOnly
          value={value}
          className={`flex-1 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm outline-none ${mono ? 'font-mono' : ''}`}
        />
        <button
          type='button'
          onClick={copy}
          className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
          aria-label={`Copy ${label}`}
        >
          {copied ? <Check className='h-3.5 w-3.5 text-emerald-500' /> : <Copy className='h-3.5 w-3.5' />}
        </button>
      </div>
    </Field>
  )
}

function SecretField({ label, value }: { label: string; value: string }) {
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const copy = () => {
    void navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }
  return (
    <Field label={label} hint='Keep this secret safe — it grants full access on behalf of the application.'>
      <div className='flex items-center gap-2'>
        <input
          readOnly
          type={revealed ? 'text' : 'password'}
          value={value}
          className='flex-1 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm font-mono outline-none'
        />
        <button
          type='button'
          onClick={() => setRevealed((r) => !r)}
          className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
          aria-label={revealed ? 'Hide secret' : 'Reveal secret'}
        >
          {revealed ? <EyeOff className='h-3.5 w-3.5' /> : <Eye className='h-3.5 w-3.5' />}
        </button>
        <button
          type='button'
          onClick={copy}
          className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
          aria-label='Copy secret'
        >
          {copied ? <Check className='h-3.5 w-3.5 text-emerald-500' /> : <Copy className='h-3.5 w-3.5' />}
        </button>
      </div>
    </Field>
  )
}

interface DurationFieldProps {
  label: string
  value: number | null
  onChange: (value: number | null) => void
}

function DurationField({ label, value, onChange }: DurationFieldProps) {
  return (
    <Field label={label} hint='Seconds. Empty = realm default.'>
      <input
        type='number'
        min={0}
        value={value ?? ''}
        onChange={(e) => {
          const raw = e.target.value
          onChange(raw === '' ? null : Number(raw))
        }}
        placeholder='Inherit'
        className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
      />
    </Field>
  )
}
