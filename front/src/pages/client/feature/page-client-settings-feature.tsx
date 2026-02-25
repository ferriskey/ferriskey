import { useGetClient, useUpdateClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router-dom'
import PageClientSettings from '../ui/page-client-settings'
import { useForm } from 'react-hook-form'
import { updateClientSchema, UpdateClientSchema } from '../schemas/update-client.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { useFormChanges } from '@/hooks/use-form-changes'

export default function PageClientSettingsFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { data: clientResponse ,refetch} = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id ?? '',
  })
  const { mutate: updateClient } = useUpdateClient()

  const form = useForm<UpdateClientSchema>({
    resolver: zodResolver(updateClientSchema),
    defaultValues: {
      clientId: clientResponse?.data.client_id ?? '',
      name: clientResponse?.data.name ?? '',
      enabled: clientResponse?.data.enabled ?? false,
      directAccessGrantsEnabled: clientResponse?.data.direct_access_grants_enabled ?? false,
    },
  })

  const hasChanges = useFormChanges(
    form,
    clientResponse && {
      clientId: clientResponse.data.client_id ?? '',
      name: clientResponse.data.name ?? '',
      enabled: clientResponse.data.enabled ?? false,
      directAccessGrantsEnabled: clientResponse.data.direct_access_grants_enabled ?? false,
    }
  )

  const handleSubmit = form.handleSubmit((values) => {
    if (!clientResponse) return

    updateClient({
      body: {
        client_id: values.clientId,
        name: values.name,
        enabled: values.enabled,
        direct_access_grants_enabled: values.directAccessGrantsEnabled,
      },
      path: {
        client_id: clientResponse.data.id,
        realm_name: realm_name ?? 'master'
      }
    })
  })

  useEffect(() => {
    if (clientResponse) {
      form.reset({
        clientId: clientResponse.data.client_id,
        name: clientResponse.data.name,
        enabled: clientResponse.data.enabled,
        directAccessGrantsEnabled: clientResponse.data.direct_access_grants_enabled,
      })
    }
  }, [clientResponse, form])

  return (
    <Form {...form}>
      <>
        {clientResponse && (
          <PageClientSettings
            client={clientResponse.data}
            form={form}
            handleSubmit={handleSubmit}
            hasChanges={hasChanges}
            refetch={refetch}
          />
        )}
      </>
    </Form>
  )
}
