import { Schemas } from '@/api/api.client'
import { useGetClientSecret } from '@/api/client.api'
import { Check, Copy, Eye, EyeOff, Loader2, RefreshCw, ShieldAlert } from 'lucide-react'
import { useState } from 'react'
import { CopyRow, Field, Section } from './primitives'

import Client = Schemas.Client

export default function CredentialsTab({ client, realm }: { client: Client; realm: string }) {
  const isConfidential = client.client_type === 'confidential'

  return (
    <div className='flex flex-col gap-6'>
      <Section title='Client ID' description='Public identifier used in OAuth / OIDC requests.'>
        <CopyRow value={client.client_id} />
      </Section>

      {isConfidential ? (
        <Section
          title='Client secret'
          description='Confidential credential used to authenticate this application to FerrisKey.'
        >
          {client.secret ? (
            <SecretField realm={realm} clientId={client.id} />
          ) : (
            <p className='text-xs text-muted-foreground'>
              The client secret is hidden for this view.
            </p>
          )}
          <div className='flex items-center justify-between gap-4 rounded-md border border-dashed border-border p-3'>
            <div>
              <p className='text-sm font-medium'>Rotate secret</p>
              <p className='text-xs text-muted-foreground mt-0.5'>
                Generate a new secret and invalidate the current one.
              </p>
            </div>
            <button
              type='button'
              disabled
              title='Coming soon'
              className='inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-2 text-sm font-medium text-muted-foreground/60 cursor-not-allowed'
            >
              <RefreshCw className='h-3.5 w-3.5' />
              Rotate
              <span className='rounded-md bg-muted px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide'>
                Soon
              </span>
            </button>
          </div>
        </Section>
      ) : (
        <Section title='Client secret' description='Public clients do not use a secret.'>
          <p className='text-xs text-muted-foreground'>
            This is a public client (SPA, native or device app). It authenticates using
            Authorization Code + PKCE rather than a client secret.
          </p>
        </Section>
      )}
    </div>
  )
}

const MASKED_SECRET = '••••••••••••••••••••••••'

function SecretField({ realm, clientId }: { realm: string; clientId: string }) {
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)

  const { data, error, isFetching } = useGetClientSecret({
    realm,
    clientId,
    enabled: revealed,
  })

  const secret = revealed ? (data?.client_secret ?? null) : null
  const status = (error as { status?: number } | null)?.status
  const forbidden = revealed && status === 403
  const failed = revealed && !!error && !forbidden

  const copy = () => {
    if (!secret) return
    void navigator.clipboard.writeText(secret)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }

  return (
    <Field
      label='Secret'
      hint='Keep this safe — it grants full access on behalf of the application. Revealing it is recorded as a security event.'
    >
      <div className='flex items-center gap-2'>
        <input
          readOnly
          type='text'
          value={secret ?? MASKED_SECRET}
          className='flex-1 rounded-md border border-border bg-muted/40 px-3 py-2 text-sm font-mono outline-none'
        />
        <button
          type='button'
          onClick={() => setRevealed((r) => !r)}
          disabled={isFetching}
          className='inline-flex h-9 items-center justify-center gap-1.5 rounded-md border border-border bg-background px-3 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-60 disabled:cursor-not-allowed'
          aria-label={revealed ? 'Hide secret' : 'Reveal secret'}
        >
          {isFetching ? (
            <Loader2 className='h-3.5 w-3.5 animate-spin' />
          ) : revealed ? (
            <EyeOff className='h-3.5 w-3.5' />
          ) : (
            <Eye className='h-3.5 w-3.5' />
          )}
          {revealed ? 'Hide' : 'Reveal'}
        </button>
        <button
          type='button'
          onClick={copy}
          disabled={!secret}
          className='inline-flex h-9 w-9 items-center justify-center rounded-md border border-border bg-background text-muted-foreground hover:text-foreground hover:bg-muted transition-colors disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-background disabled:hover:text-muted-foreground'
          aria-label='Copy secret'
        >
          {copied ? (
            <Check className='h-3.5 w-3.5 text-emerald-500' />
          ) : (
            <Copy className='h-3.5 w-3.5' />
          )}
        </button>
      </div>

      {forbidden && (
        <p className='flex items-start gap-1.5 text-xs text-amber-600 dark:text-amber-500'>
          <ShieldAlert className='h-3.5 w-3.5 shrink-0 mt-0.5' />
          You need the manage-clients permission to reveal this secret. Ask a realm administrator
          for access.
        </p>
      )}
      {failed && (
        <p className='flex items-start gap-1.5 text-xs text-destructive'>
          <ShieldAlert className='h-3.5 w-3.5 shrink-0 mt-0.5' />
          The secret could not be revealed. Please try again.
        </p>
      )}
    </Field>
  )
}
