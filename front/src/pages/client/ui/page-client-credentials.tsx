import { InputText } from '@/components/ui/input-text'
import { Schemas } from '@/api/api.client.ts'
import BlockContent from '@/components/ui/block-content'
import Client = Schemas.Client

export interface PageClientCredentialsProps {
  client: Client
}

export default function PageClientCredentials({ client }: PageClientCredentialsProps) {
  return (
    <div className='flex w-full md:w-1/2 lg:w-1/3'>
      <BlockContent title='Client Credentials'>
        <div className='flex flex-col gap-4'>
          <InputText
            label='Client ID'
            name='client_id'
            value={client.client_id}
            disabled
          />
          <InputText
            label='Client Secret'
            name='client_secret'
            type='password'
            value={client.secret ?? ''}
            disabled
            togglePasswordVisibility={true}
          />
        </div>
      </BlockContent>
    </div>
  )
}
