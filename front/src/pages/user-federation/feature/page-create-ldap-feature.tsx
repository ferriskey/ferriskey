import { useNavigate, useParams } from 'react-router'
import { USER_FEDERATION_OVERVIEW_URL, USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createLdapProviderSchema, CreateLdapProviderSchema } from '../schemas/ldap-provider.schema'
import { Form } from '@/components/ui/form'
import LdapFormUi from '../ui/ldap-form-ui'

export default function PageCreateLdapFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()

  const form = useForm<CreateLdapProviderSchema>({
    resolver: zodResolver(createLdapProviderSchema),
    mode: 'onChange',
    defaultValues: {
      type: 'LDAP',
      name: '',
      priority: 'Secondary',
      enabled: true,
      connectionUrl: '',
      baseDn: '',
      bindDn: '',
      bindPassword: '',
      userSearchFilter: '(objectClass=person)',
      syncInterval: 3600,
      useTls: false,
    },
  })

  const handleTypeChange = (newType: 'LDAP' | 'Kerberos') => {
    if (newType === 'Kerberos') {
      navigate(`${USER_FEDERATION_URL(realm_name)}/kerberos/create`)
    }
  }

  const handleBack = () => {
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  const handleSubmit = (data: CreateLdapProviderSchema) => {
    console.log('Creating LDAP provider:', data)
    toast.success('LDAP Provider created successfully')
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)}>
        <LdapFormUi
          form={form}
          handleBack={handleBack}
          handleSubmit={form.handleSubmit(handleSubmit)}
          onTypeChange={handleTypeChange}
        />
      </form>
    </Form>
  )
}
