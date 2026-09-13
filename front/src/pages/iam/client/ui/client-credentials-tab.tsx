import { useState } from 'react'
import { Check, Copy, Eye, EyeOff, Loader2, ShieldAlert } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'

import Client = Schemas.Client

export interface ClientCredentialsTabProps {
  client: Client
  revealed: boolean
  secret: string | null
  isFetching: boolean
  forbidden: boolean
  failed: boolean
  onToggleReveal: () => void
}

const MASKED_SECRET = '••••••••••••••••••••••••'

function CopyButton({ value, disabled }: { value: string; disabled?: boolean }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Button
      type='button'
      variant='outline'
      size='icon'
      disabled={disabled}
      aria-label='Copy'
      onClick={() => void handleCopy()}
    >
      {copied ? <Check className='size-4 text-fk-success' /> : <Copy className='size-4' />}
    </Button>
  )
}

function Fingerprint({ value, muted }: { value: string; muted?: boolean }) {
  return (
    <code
      className={cn(
        'flex h-9 min-w-0 flex-1 items-center overflow-x-auto rounded-md border border-fk-line bg-neutral-50 dark:bg-fk-surface px-2.5 font-mono-ui text-xs',
        muted ? 'text-neutral-400 dark:text-neutral-500' : 'text-neutral-700 dark:text-neutral-300'
      )}
    >
      {value}
    </code>
  )
}

export default function ClientCredentialsTab({
  client,
  revealed,
  secret,
  isFetching,
  forbidden,
  failed,
  onToggleReveal,
}: ClientCredentialsTabProps) {
  return (
    <Section
      title='Client credentials'
      description='What this client presents to prove who it is.'
    >
      <FieldRow
        label='Client ID'
        description='The unique identifier used to authenticate this client.'
      >
        <div className='flex max-w-lg items-center gap-2'>
          <Fingerprint value={client.client_id} />
          <CopyButton value={client.client_id} />
        </div>
      </FieldRow>

      <FieldRow
        label='Client secret'
        description='The secret used for confidential client authentication. Revealing it is recorded as a security event.'
      >
        <div className='max-w-lg space-y-2'>
          <div className='flex items-center gap-2'>
            <Fingerprint value={secret ?? MASKED_SECRET} muted={!secret} />
            <Button
              type='button'
              variant='outline'
              size='sm'
              disabled={isFetching}
              aria-label={revealed ? 'Hide client secret' : 'Reveal client secret'}
              onClick={onToggleReveal}
            >
              {isFetching ? (
                <Loader2 className='size-4 animate-spin' />
              ) : revealed ? (
                <EyeOff className='size-4' />
              ) : (
                <Eye className='size-4' />
              )}
              {revealed ? 'Hide' : 'Reveal'}
            </Button>
            <CopyButton value={secret ?? ''} disabled={!secret} />
          </div>

          {forbidden && (
            <p className='flex items-start gap-1.5 text-xs text-fk-amber'>
              <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
              You need the manage-clients permission to reveal this secret. Ask a realm
              administrator for access.
            </p>
          )}
          {failed && (
            <p className='flex items-start gap-1.5 text-xs text-fk-danger'>
              <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
              The secret could not be revealed. Please try again.
            </p>
          )}
          <p className='text-xs text-neutral-400 dark:text-neutral-500'>
            Rotating the secret is not exposed by the administration API yet.
          </p>
        </div>
      </FieldRow>
    </Section>
  )
}
