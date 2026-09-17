import { useState } from 'react'
import { Check, Copy, Eye, EyeOff, Loader2, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation('client')
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
      aria-label={t('credentials.copy')}
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
  const { t } = useTranslation('client')

  return (
    <Section title={t('credentials.title')} description={t('credentials.description')}>
      <FieldRow
        label={t('credentials.client_id.label')}
        description={t('credentials.client_id.description')}
      >
        <div className='flex max-w-lg items-center gap-2'>
          <Fingerprint value={client.client_id} />
          <CopyButton value={client.client_id} />
        </div>
      </FieldRow>

      <FieldRow
        label={t('credentials.secret.label')}
        description={t('credentials.secret.description')}
      >
        <div className='max-w-lg space-y-2'>
          <div className='flex items-center gap-2'>
            <Fingerprint value={secret ?? MASKED_SECRET} muted={!secret} />
            <Button
              type='button'
              variant='outline'
              size='sm'
              disabled={isFetching}
              aria-label={revealed ? t('credentials.secret.hide_label') : t('credentials.secret.reveal_label')}
              onClick={onToggleReveal}
            >
              {isFetching ? (
                <Loader2 className='size-4 animate-spin' />
              ) : revealed ? (
                <EyeOff className='size-4' />
              ) : (
                <Eye className='size-4' />
              )}
              {revealed ? t('credentials.secret.hide') : t('credentials.secret.reveal')}
            </Button>
            <CopyButton value={secret ?? ''} disabled={!secret} />
          </div>

          {forbidden && (
            <p className='flex items-start gap-1.5 text-xs text-fk-amber'>
              <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
              {t('credentials.secret.forbidden')}
            </p>
          )}
          {failed && (
            <p className='flex items-start gap-1.5 text-xs text-fk-danger'>
              <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
              {t('credentials.secret.failed')}
            </p>
          )}
          <p className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('credentials.secret.rotation_hint')}
          </p>
        </div>
      </FieldRow>
    </Section>
  )
}
