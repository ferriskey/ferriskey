import { Schemas } from '@/api/api.client'
import { Check, Copy, Eye, EyeOff, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { CopyRow, Field, Section } from './primitives'

import Client = Schemas.Client

export default function CredentialsTab({ client }: { client: Client }) {
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
            <SecretField value={client.secret} />
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

function SecretField({ value }: { value: string }) {
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const copy = () => {
    void navigator.clipboard.writeText(value)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1500)
  }
  return (
    <Field label='Secret' hint='Keep this safe — it grants full access on behalf of the application.'>
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
