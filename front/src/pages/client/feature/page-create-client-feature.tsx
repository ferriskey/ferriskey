import { useCreateClient } from '@/api/client.api.ts'
import { useForm } from 'react-hook-form'
import { CreateClientSchema, createClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router.ts'
import { useEffect, useMemo } from 'react'
import { CLIENTS_URL, OVERVIEW_URL } from '@/routes/sub-router/client.router.ts'
import { toast } from 'sonner'
import PageCreateClient from '@/pages/client/ui/page-create-client.tsx'
import { Form } from '@/components/ui/form.tsx'

export default function PageCreateClientFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate: createClient, data: responseCreateClient } = useCreateClient()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema),
    defaultValues: {
      clientId: '',
      name: '',
      enabled: false,
      protocol: 'openid-connect',
      clientAuthentication: false
    }
  })

  const url = useMemo(() => {
    if (!realm_name) return ''

    return `${CLIENTS_URL(realm_name)}${OVERVIEW_URL}`
  }, [realm_name])

  const onSubmit = () => {
    const data = form.getValues()

    if (!realm_name) return

    createClient({
      path: {
        realm_name,
      },
      body: {
        ...data,
        client_type: data.clientAuthentication ? 'confidential' : 'public',
        public_client: !data.clientAuthentication,
        service_account_enabled: !!data.clientAuthentication,
      }
    })
  }

  const handleBack = () => {
    navigate(url)
  }

  useEffect(() => {
    if (responseCreateClient) {
      toast.success('The client has been successfully created')
      navigate(url)
    }
  }, [responseCreateClient, navigate, url])

  const formIsValid = form.formState.isValid && form.formState.isDirty

  return (
    <Form {...form}>
      <PageCreateClient
        form={form}
        handleBack={handleBack}
        handleSubmit={onSubmit}
        formIsValid={formIsValid}
      />
    </Form>
  )
}
