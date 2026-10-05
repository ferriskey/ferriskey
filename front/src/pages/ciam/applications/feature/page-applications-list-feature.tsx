import { useParams } from 'react-router'
import {
  APPLICATION_FILTER_KEYS,
  useClientCount,
  useGetClients,
  type ClientsQuery,
} from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useCreatePicker, usePagedListing } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  CONSOLE_APPLICATION_CREATE_URL,
  CONSOLE_APPLICATION_URL,
} from '../application-routes'
import { type ApplicationType } from '../application-types'
import PageApplicationsList from '../ui/page-applications-list'

import Client = Schemas.Client

const ALERT_PREVIEW = 5

const displayName = (application: Client) => application.name || application.client_id

export default function PageApplicationsListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(APPLICATION_FILTER_KEYS)
  const { data: response, isLoading } = useGetClients({
    realm,
    query: listing.apiQuery as ClientsQuery,
    keepPrevious: true,
  })
  const total = useClientCount({ realm })
  const native = useClientCount({ realm, filter: { application_type: 'native' } })
  const spa = useClientCount({ realm, filter: { application_type: 'spa' } })
  const web = useClientCount({ realm, filter: { application_type: 'web' } })
  const m2m = useClientCount({ realm, filter: { application_type: 'm2m' } })
  const device = useClientCount({ realm, filter: { application_type: 'device' } })
  const { data: missingCallback } = useGetClients({
    realm,
    query: {
      service_account_enabled: false,
      oauth_device_code_grant_enabled: false,
      has_redirect_uris: false,
      limit: ALERT_PREVIEW,
    },
  })

  const picker = useCreatePicker()

  return (
    <PageApplicationsList
      applications={response?.data ?? []}
      pagination={response?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        native: native.count,
        spa: spa.count,
        web: web.count,
        m2m: m2m.count,
        device: device.count,
      }}
      missingCallback={{
        total: missingCallback?.metadata.total ?? 0,
        names: (missingCallback?.data ?? []).map(displayName),
      }}
      pickerOpen={picker.open}
      onPickerOpenChange={picker.setOpen}
      createUrl={(type: ApplicationType) =>
        `${CONSOLE_APPLICATION_CREATE_URL(realm)}?type=${type}`
      }
      applicationHref={(application: Client) =>
        `${CONSOLE_APPLICATION_URL(realm, application.id)}/quickstart`
      }
    />
  )
}
