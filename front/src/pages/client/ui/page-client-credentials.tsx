import { useState } from 'react'
import { InputText } from '@/components/ui/input-text'
import { Copy, Check, Eye, EyeOff, Loader2, ShieldAlert } from 'lucide-react'
import type { Schemas } from '@/api/api.client.ts'
import { useGetClientSecret } from '@/api/client.api.ts'
type Client = Schemas.Client
export interface PageClientCredentialsProps {
  client: Client
  realm: string
}

const MASKED_SECRET = '••••••••••••••••••••••••'

function CopyButton({ value, disabled = false }: { value: string; disabled?: boolean }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <button
      type='button'
      onClick={handleCopy}
      disabled={disabled}
      className='shrink-0 min-h-[52px] w-11 flex items-center justify-center rounded-md border border-input bg-background text-muted-foreground transition-colors hover:border-ring hover:text-foreground disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-input disabled:hover:text-muted-foreground'
    >
      {copied ? (
        <Check className='h-4 w-4 text-emerald-500' />
      ) : (
        <Copy className='h-4 w-4' />
      )}
    </button>
  )
}

export default function PageClientCredentials({ client, realm }: PageClientCredentialsProps) {
  const [revealed, setRevealed] = useState(false)

  const { data, error, isFetching } = useGetClientSecret({
    realm,
    clientId: client.id,
    enabled: revealed,
  })

  const secret = revealed ? (data?.client_secret ?? null) : null
  const status = (error as { status?: number } | null)?.status
  const forbidden = revealed && status === 403
  const failed = revealed && !!error && !forbidden

  return (
    <div className='flex flex-col gap-8'>
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Authentication credentials</p>
          <h2 className='text-base font-semibold'>Client Credentials</h2>
        </div>

        {/* Client ID */}
        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Client ID</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              The unique identifier used to authenticate this client.
            </p>
          </div>
          <div className='w-1/2 flex items-center gap-2'>
            <InputText
              label='Client ID'
              name='client_id'
              value={client.client_id}
              className='flex-1'
              disabled
            />
            <CopyButton value={client.client_id} />
          </div>
        </div>

        {/* Client Secret */}
        <div className='flex items-start justify-between py-4 border-t'>
          <div className='w-1/3'>
            <p className='text-sm font-medium'>Client Secret</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              The secret used for confidential client authentication. Revealing it is recorded as a
              security event.
            </p>
          </div>
          <div className='w-1/2 flex flex-col gap-2'>
            <div className='flex items-center gap-2'>
              <InputText
                label='Client Secret'
                name='client_secret'
                value={secret ?? MASKED_SECRET}
                className='flex-1'
                disabled
              />
              <button
                type='button'
                onClick={() => setRevealed((r) => !r)}
                disabled={isFetching}
                aria-label={revealed ? 'Hide client secret' : 'Reveal client secret'}
                className='shrink-0 min-h-[52px] px-3 flex items-center justify-center gap-1.5 rounded-md border border-input bg-background text-sm font-medium text-muted-foreground transition-colors hover:border-ring hover:text-foreground disabled:opacity-60 disabled:cursor-not-allowed'
              >
                {isFetching ? (
                  <Loader2 className='h-4 w-4 animate-spin' />
                ) : revealed ? (
                  <EyeOff className='h-4 w-4' />
                ) : (
                  <Eye className='h-4 w-4' />
                )}
                {revealed ? 'Hide' : 'Reveal'}
              </button>
              <CopyButton value={secret ?? ''} disabled={!secret} />
            </div>

            {forbidden && (
              <p className='flex items-start gap-1.5 text-xs text-amber-600 dark:text-amber-500'>
                <ShieldAlert className='h-3.5 w-3.5 shrink-0 mt-0.5' />
                You need the manage-clients permission to reveal this secret. Ask a realm
                administrator for access.
              </p>
            )}
            {failed && (
              <p className='flex items-start gap-1.5 text-xs text-destructive'>
                <ShieldAlert className='h-3.5 w-3.5 shrink-0 mt-0.5' />
                The secret could not be revealed. Please try again.
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
