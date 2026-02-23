import { InputText } from '@/components/ui/input-text'
import { Schemas } from '@/api/api.client.ts'
import Client = Schemas.Client

export interface PageClientCredentialsProps {
  client: Client
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
          <div className='w-1/2'>
            <InputText
              label='Client ID'
              name='client_id'
              value={client.client_id}
              disabled
            />
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
          <div className='w-1/2'>
            <InputText
              label='Client Secret'
              name='client_secret'
              type='password'
              value={client.secret ?? ''}
              disabled
              togglePasswordVisibility={true}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
