import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useCreatePicker } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { CLIENT_CREATE_URL, CLIENT_URL } from '../client-routes'
import { type ClientProtocol } from '../client-choices'
import PageClientsOverview from '../ui/page-clients-overview'

import Client = Schemas.Client

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: clientsResponse, isLoading } = useGetClients({ realm })
  const clients = useMemo(() => clientsResponse?.data ?? [], [clientsResponse])

  const picker = useCreatePicker()

  return (
    <PageClientsOverview
      clients={clients}
      isLoading={isLoading}
      pickerOpen={picker.open}
      onPickerOpenChange={picker.setOpen}
      createUrl={(protocol: ClientProtocol) =>
        `${CLIENT_CREATE_URL(realm)}?protocol=${protocol}`
      }
      clientHref={(client: Client) => `${CLIENT_URL(realm, client.id)}/settings`}
    />
  )
}
