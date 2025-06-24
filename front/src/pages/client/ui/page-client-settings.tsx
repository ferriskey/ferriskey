import { Client } from '@/api/api.interface'
import BlockContent from '@/components/ui/block-content'
import { InputText } from '@/components/ui/input-text'
import ManageRedirectUris from '../components/manage-redirect-uris'

export interface PageClientSettingsProps {
  client: Client
}

export default function PageClientSettings({ client }: PageClientSettingsProps) {
  return (
    <div>
      <div className="flex flex-col gap-4">
        <BlockContent title="General Settings" className="w-full md:w-2/3 2xl:w-1/3">
          <div className="flex flex-col gap-4">
            <InputText name="client_name" label="Client Name" value={client.name} disabled />

            <InputText name="client_id" label="Client ID" value={client.client_id} disabled />
          </div>
        </BlockContent>

        <BlockContent title="Access Settings" className="w-full md:w-2/3 2xl:w-1/3">
          <div>
            <ManageRedirectUris redirectUris={client.redirect_uris ?? []} />
          </div>
        </BlockContent>
      </div>
    </div>
  )
}
