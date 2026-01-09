import { useNavigate, useParams } from 'react-router'
import { USER_FEDERATION_OVERVIEW_URL, USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createLdapProviderSchema, CreateLdapProviderSchema } from '../schemas/ldap-provider.schema'
import { Form } from '@/components/ui/form'
import LdapFormUi from '../ui/ldap-form-ui'
import { useGetUserFederation, useUpdateUserFederation } from '@/api/user-federation.api'
import { useEffect } from 'react'

export default function PageDetailLdapFeature() {
  const navigate = useNavigate()
  const { realm_name, id } = useParams<RouterParams & { id: string }>()
  const { data: providerData, isLoading } = useGetUserFederation(realm_name || '', id || '')
  const { mutateAsync: updateProvider } = useUpdateUserFederation()

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

  useEffect(() => {
    if (providerData) {
      const provider = providerData
      const config = provider.config as Record<string, unknown>

      const mapPriority = (p: number) => {
        if (p === 0) return 'Primary'
        if (p === 10) return 'Secondary'
        if (p === 20) return 'Development'
        return 'Legacy'
      }

      form.reset({
        type: 'LDAP',
        name: provider.name,
        priority: mapPriority(provider.priority) as 'Primary' | 'Secondary' | 'Development' | 'Legacy',
        enabled: provider.enabled,
        connectionUrl: (config?.connectionUrl as string) || '',
        baseDn: (config?.baseDn as string) || '',
        bindDn: (config?.bindDn as string) || '',
        bindPassword: (config?.bindPassword as string) || '',
        userSearchFilter: (config?.userSearchFilter as string) || '(objectClass=person)',
        syncInterval: (provider.sync_interval_minutes || 60) * 60,
        useTls: (config?.useTls as boolean) || false,
      })
    }
  }, [providerData, form])

  const handleTypeChange = (newType: 'LDAP' | 'Kerberos') => {
    if (newType === 'Kerberos') {
      navigate(`${USER_FEDERATION_URL(realm_name)}/kerberos/${id}`)
    }
  }

  const handleBack = () => {
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  const handleSubmit = async (data: CreateLdapProviderSchema) => {
    if (!realm_name || !id) return

    try {
      await updateProvider({
        path: { realm_name, id },
        body: {
          name: data.name,
          enabled: data.enabled,
          provider_type: 'Ldap',
          priority: data.priority === 'Primary' ? 0 : data.priority === 'Secondary' ? 10 : 20,
          sync_mode: 'Import',
          sync_enabled: true,
          sync_interval_minutes: Math.floor(data.syncInterval / 60),
          config: {
            connectionUrl: data.connectionUrl,
            baseDn: data.baseDn,
            bindDn: data.bindDn,
            bindPassword: data.bindPassword,
            userSearchFilter: data.userSearchFilter,
            useTls: data.useTls,
          }
        }
      })
      toast.success('LDAP Provider updated successfully')
      navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
    } catch (error) {
      console.error('Failed to update provider', error)
      toast.error('Failed to update LDAP provider')
    }
  }

  if (isLoading) {
    return <div className='p-4 text-center'>Loading provider...</div>
  }

  if (!providerData) {
    return <div className='p-4 text-center'>Provider not found</div>
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)}>
        <LdapFormUi
          form={form}
          handleBack={handleBack}
          handleSubmit={form.handleSubmit(handleSubmit)}
          onTypeChange={handleTypeChange}
          isEditMode={true}
        />
      </form>
    </Form>
  )
}
