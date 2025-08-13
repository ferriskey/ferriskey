import { useGetClient, useUpdateClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router'
import PageClientSettings from '../ui/page-client-settings'
import { useForm } from 'react-hook-form'
import { updateClientSchema, UpdateClientSchema } from '../schemas/update-client.schema'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useEffect } from 'react'
import { useFormChanges } from '@/hooks/use-form-changes'

export default function PageClientSettingsFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { data: clientResponse } = useGetClient({
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
    },
  })

  const hasChanges = useFormChanges(
    form,
    clientResponse && {
      clientId: clientResponse.data.client_id ?? '',
      name: clientResponse.data.name ?? '',
      enabled: clientResponse.data.enabled ?? false,
    }
  )

  const handleSubmit = form.handleSubmit((values) => {
    if (!clientResponse) return

    updateClient({
      realm: realm_name,
      clientId: clientResponse.data.id,
      payload: values,
    })
  })

  useEffect(() => {
    if (clientResponse) {
      form.reset({
        clientId: clientResponse.data.client_id,
        name: clientResponse.data.name,
        enabled: clientResponse.data.enabled,
      })
    }
  }, [clientResponse])

  return (
    <Form {...form}>
      <>
        {clientResponse && (
          <PageClientSettings
            client={clientResponse.data}
            form={form}
            handleSubmit={handleSubmit}
            hasChanges={hasChanges}
          />
        )}
      </>
    </Form>
  )
}
