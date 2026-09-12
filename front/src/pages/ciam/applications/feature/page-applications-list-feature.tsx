import { useMemo } from 'react'
import { useParams } from 'react-router'
import { useGetClients } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useCreatePicker } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  CONSOLE_APPLICATION_CREATE_URL,
  CONSOLE_APPLICATION_URL,
} from '../application-routes'
import { type ApplicationType } from '../application-types'
import PageApplicationsList from '../ui/page-applications-list'

import Client = Schemas.Client

export default function PageApplicationsListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: clientsResponse, isLoading } = useGetClients({ realm })
  const applications = useMemo(() => clientsResponse?.data ?? [], [clientsResponse])

  const picker = useCreatePicker()

  return (
    <PageApplicationsList
      applications={applications}
      isLoading={isLoading}
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
