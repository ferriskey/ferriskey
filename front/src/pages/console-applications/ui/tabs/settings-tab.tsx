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
import { Loader2, Plus, Trash2, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import { inferApplicationType } from '../../types'
import { Field, Section } from './primitives'

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

const isValidUrl = (s: string) => /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\/.+/.test(s.trim())

interface Props {
  client: Client
  isSaving: boolean
  uriPending: boolean
  onSave: (values: ApplicationSettingsValues) => Promise<void>
  onDelete: () => void
  onAddRedirectUri: (value: string) => void
  onDeleteRedirectUri: (redirectUriId: string) => void
}

export default function SettingsTab({
  client,
  isSaving,
  uriPending,
  onSave,
  onDelete,
  onAddRedirectUri,
  onDeleteRedirectUri,
}: Props) {
  const [values, setValues] = useState<ApplicationSettingsValues>(() => settingsFromClient(client))
  const [newUri, setNewUri] = useState('')

  const baseline = useMemo(() => settingsFromClient(client), [client])
  const hasChanges = useMemo(
    () => JSON.stringify(values) !== JSON.stringify(baseline),
    [values, baseline],
  )

  const appType = inferApplicationType(client)
  const showRedirectUris = appType !== 'm2m' && appType !== 'device'

  const set = <K extends keyof ApplicationSettingsValues>(
    key: K,
    value: ApplicationSettingsValues[K],
  ) => setValues((prev) => ({ ...prev, [key]: value }))

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
    <form onSubmit={handleSubmit} className='flex flex-col gap-6'>
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

      <Section
        title='Token lifetimes'
        description='Override realm defaults for this application. Leave empty to inherit the realm setting.'
      >
        <div className='grid grid-cols-1 sm:grid-cols-2 gap-4'>
          <DurationField label='Access token' value={values.accessTokenLifetime} onChange={(v) => set('accessTokenLifetime', v)} />
          <DurationField label='Refresh token' value={values.refreshTokenLifetime} onChange={(v) => set('refreshTokenLifetime', v)} />
          <DurationField label='ID token' value={values.idTokenLifetime} onChange={(v) => set('idTokenLifetime', v)} />
          <DurationField label='Temporary token' value={values.temporaryTokenLifetime} onChange={(v) => set('temporaryTokenLifetime', v)} />
        </div>
      </Section>

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
                <AlertDialogAction onClick={onDelete} className='bg-red-500 text-white hover:bg-red-500/90'>
                  Delete application
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </Section>

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
