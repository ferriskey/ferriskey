import { useState } from 'react'
import { InputText } from '@/components/ui/input-text'
import { Copy, Check } from 'lucide-react'
import { Schemas } from '@/api/api.client.ts'
import Client = Schemas.Client

export interface PageClientCredentialsProps {
  client: Client
}

function CopyButton({ value }: { value: string }) {
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
      className='shrink-0 min-h-[52px] w-11 flex items-center justify-center rounded-md border border-input bg-background text-muted-foreground transition-colors hover:border-ring hover:text-foreground'
    >
      {copied ? (
        <Check className='h-4 w-4 text-emerald-500' />
      ) : (
        <Copy className='h-4 w-4' />
      )}
    </button>
  )
}

export default function PageClientCredentials({ client }: PageClientCredentialsProps) {
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
              The secret used for confidential client authentication.
            </p>
          </div>
          <div className='w-1/2 flex items-center gap-2'>
            <InputText
              label='Client Secret'
              name='client_secret'
              type='password'
              value={client.secret ?? ''}
              className='flex-1'
              disabled
              togglePasswordVisibility={true}
            />
            <CopyButton value={client.secret ?? ''} />
          </div>
        </div>
      </div>
    </div>
  )
}
