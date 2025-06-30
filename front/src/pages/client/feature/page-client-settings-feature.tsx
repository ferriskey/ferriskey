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
  const { data } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id ?? '',
  })
  const { mutate: updateClient } = useUpdateClient()

  const form = useForm<UpdateClientSchema>({
    resolver: zodResolver(updateClientSchema),
    defaultValues: {
      clientId: data?.client_id ?? '',
      name: data?.name ?? '',
      enabled: data?.enabled ?? false,
    },
  })

  const hasChanges = useFormChanges(
    form,
    data && {
      clientId: data.client_id ?? '',
      name: data.name ?? '',
      enabled: data.enabled ?? false,
    }
  )

  const handleSubmit = form.handleSubmit((values) => {
    if (!data) return

    updateClient({
      realm: realm_name,
      clientId: data.id,
      payload: values,
    })
  })

  useEffect(() => {
    if (data) {
      form.reset({
        clientId: data.client_id,
        name: data.name,
        enabled: data.enabled,
      })
    }
  }, [data])

  return (
    <Form {...form}>
      <>
        {data && (
          <PageClientSettings
            client={data}
            form={form}
            handleSubmit={handleSubmit}
            hasChanges={hasChanges}
          />
        )}
      </>
    </Form>
  )
}
