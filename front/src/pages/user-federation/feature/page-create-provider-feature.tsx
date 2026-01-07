import { useNavigate, useParams } from 'react-router'
import PageCreateProvider from '../ui/page-create-provider'
import { USER_FEDERATION_OVERVIEW_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createProviderSchema, CreateProviderSchema } from '../schemas/create-provider.schema'
import { Form } from '@/components/ui/form'

export default function PageCreateProviderFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const form = useForm<CreateProviderSchema>({
    resolver: zodResolver(createProviderSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      type: 'LDAP',
      priority: 'Secondary',
      enabled: true,
      connectionUrl: '',
      baseDn: '',
      bindDn: '',
      bindPassword: '',
      userSearchBase: '',
      userSearchFilter: '(objectClass=person)',
      syncInterval: 3600,
    },
  })

  const handleBack = () => {
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  const handleSubmit = () => {
    const data = form.getValues()
    // TODO: Implement API call to create provider
    console.log('Creating provider:', data)
    toast.success('Provider created successfully')
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  return (
    <Form {...form}>
      <PageCreateProvider
        form={form}
        handleBack={handleBack}
        handleSubmit={handleSubmit}
        formIsValid={form.formState.isValid}
      />
    </Form>
  )
}
