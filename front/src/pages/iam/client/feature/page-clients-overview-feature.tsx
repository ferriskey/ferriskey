import { useParams } from 'react-router'
import {
  CLIENT_FILTER_KEYS,
  useClientCount,
  useGetClients,
  type ClientsQuery,
} from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useCreatePicker, usePagedListing } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { CLIENT_CREATE_URL, CLIENT_URL } from '../client-routes'
import { type ClientProtocol } from '../client-choices'
import PageClientsOverview from '../ui/page-clients-overview'

import Client = Schemas.Client

const ALERT_PREVIEW = 5

const clientLabel = (client: Client) => client.name || client.client_id

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(CLIENT_FILTER_KEYS)
  const { data: clientsResponse, isLoading } = useGetClients({
    realm,
    query: listing.apiQuery as ClientsQuery,
    keepPrevious: true,
  })
  const total = useClientCount({ realm })
  const active = useClientCount({ realm, filter: { enabled: true } })
  const publicClients = useClientCount({ realm, filter: { public_client: true } })
  const confidential = useClientCount({ realm, filter: { public_client: false } })
  const { data: withoutRedirect } = useGetClients({
    realm,
    query: { has_redirect_uris: false, limit: ALERT_PREVIEW },
  })
  const { data: inMaintenance } = useGetClients({
    realm,
    query: { maintenance_enabled: true, limit: ALERT_PREVIEW },
  })

  const picker = useCreatePicker()

  return (
    <PageClientsOverview
      clients={clientsResponse?.data ?? []}
      pagination={clientsResponse?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        active: active.count,
        public: publicClients.count,
        confidential: confidential.count,
      }}
      withoutRedirect={{
        total: withoutRedirect?.metadata.total ?? 0,
        names: (withoutRedirect?.data ?? []).map(clientLabel),
      }}
      inMaintenance={{
        total: inMaintenance?.metadata.total ?? 0,
        names: (inMaintenance?.data ?? []).map(clientLabel),
      }}
      pickerOpen={picker.open}
      onPickerOpenChange={picker.setOpen}
      createUrl={(protocol: ClientProtocol) =>
        `${CLIENT_CREATE_URL(realm)}?protocol=${protocol}`
      }
      clientHref={(client: Client) => `${CLIENT_URL(realm, client.id)}/settings`}
    />
  )
}
