import { useNavigate, useParams } from 'react-router'
import { USER_FEDERATION_OVERVIEW_URL, USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createLdapProviderSchema, CreateLdapProviderSchema } from '../schemas/ldap-provider.schema'
import { Form } from '@/components/ui/form'
import LdapFormUi from '../ui/ldap-form-ui'
import { useCreateUserFederation } from '@/api/user-federation.api'

export default function PageCreateLdapFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { mutateAsync: createProvider } = useCreateUserFederation()

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

  const handleSubmit = async (data: CreateLdapProviderSchema) => {
    if (!realm_name) return

    try {
      await createProvider({
        path: { realm_name },
        body: {
          name: data.name,
          enabled: data.enabled,
          provider_type: 'Ldap',
          priority: data.priority === 'Primary' ? 0 : data.priority === 'Secondary' ? 10 : 20, // Simple mapping
          sync_mode: 'Import', // Defaulting to full
          sync_enabled: true,
          sync_interval_minutes: Math.floor(data.syncInterval / 60),
          config: {
            connectionUrl: data.connectionUrl,
            baseDn: data.baseDn,
            bindDn: data.bindDn,
            bindPassword: data.bindPassword,
            userSearchFilter: data.userSearchFilter,
            useTls: data.useTls,
            // Add other specific config fields if needed
          }
        }
      })
      toast.success('LDAP Provider created successfully')
      navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
    } catch (error) {
      console.error('Failed to create provider', error)
      toast.error('Failed to create LDAP provider')
    }
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
