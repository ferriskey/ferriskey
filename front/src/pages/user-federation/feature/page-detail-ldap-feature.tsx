import { useNavigate, useParams } from 'react-router-dom'
import { USER_FEDERATION_OVERVIEW_URL, USER_FEDERATION_URL } from '@/routes/sub-router/user-federation.router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { createLdapProviderSchema, CreateLdapProviderSchema } from '../schemas/ldap-provider.schema'
import { Form } from '@/components/ui/form'
import LdapFormUi from '../ui/ldap-form-ui'
import { useGetUserFederation, useUpdateUserFederation, useTestUserFederationConnection, useSyncUsers } from '@/api/user-federation.api'
import { useFormChanges } from '@/hooks/use-form-changes'
import { Schemas } from '@/api/api.client'
import { useConfettiFireworks } from '@/hooks/use-confetti-fireworks'
import { useEffect } from 'react'

interface LdapConnection {
  server_url: string
  port: number
  use_tls: boolean
  use_starttls: boolean
  connection_timeout_seconds: number
}

interface LdapBind {
  bind_dn: string
  bind_password_encrypted: string
}

interface LdapSearch {
  base_dn: string
  user_search_filter: string
}

interface LdapAttributes {
  username: string
  email: string
  first_name: string
  last_name: string
  external_id_attribute?: string
}

interface LdapConfig {
  connection?: LdapConnection
  bind?: LdapBind
  search?: LdapSearch
  attributes?: LdapAttributes
}

const mapPriority = (p: number) => {
  if (p === 0) return 'Primary'
  if (p === 10) return 'Secondary'
  if (p === 20) return 'Development'
  return 'Legacy'
}

interface LdapDetailFormProps {
  providerData: Schemas.ProviderResponse
  realm_name: string
  id: string
}

function LdapDetailForm({ providerData, realm_name, id }: LdapDetailFormProps) {
  const navigate = useNavigate()
  const { mutateAsync: updateProvider } = useUpdateUserFederation()
  const { mutateAsync: testConnection, isPending: isTestingConnection, isSuccess: isTestingConnectionIsSuccess } = useTestUserFederationConnection()
  const { mutateAsync: syncUsers, isPending: isSyncingUsers } = useSyncUsers()
  const { fire } = useConfettiFireworks()

  const config = providerData.config as LdapConfig

  // Construct connection URL from nested config
  const protocol = config?.connection?.use_tls ? 'ldaps' : 'ldap'
  const server = config?.connection?.server_url || ''
  const port = config?.connection?.port || (config?.connection?.use_tls ? 636 : 389)
  const connectionUrl = server ? `${protocol}://${server}:${port}` : ''

  // Decode password from base64 (backend sanitizes it to "********" in responses)
  const decodedPassword = (() => {
    const encrypted = config?.bind?.bind_password_encrypted
    if (!encrypted || encrypted === '********') {
      return ''
    }
    try {
      return atob(encrypted)
    } catch {
      // If decoding fails, assume it's already plaintext or invalid
      return encrypted
    }
  })()

  const defaultValues: CreateLdapProviderSchema = {
    type: 'LDAP',
    name: providerData.name,
    priority: mapPriority(providerData.priority) as 'Primary' | 'Secondary' | 'Development' | 'Legacy',
    enabled: providerData.enabled,
    connectionUrl: connectionUrl,
    baseDn: config?.search?.base_dn || '',
    bindDn: config?.bind?.bind_dn || '',
    bindPassword: decodedPassword,
    userSearchFilter: config?.search?.user_search_filter || '(objectClass=person)',
    syncInterval: (providerData.sync_interval_minutes || 60) * 60,
    useTls: config?.connection?.use_tls || false,
  }

  const form = useForm<CreateLdapProviderSchema>({
    resolver: zodResolver(createLdapProviderSchema),
    mode: 'onChange',
    defaultValues,
  })

  const hasChanges = useFormChanges(form, defaultValues)

  const handleTypeChange = (newType: 'LDAP' | 'Kerberos') => {
    if (newType === 'Kerberos') {
      navigate(`${USER_FEDERATION_URL(realm_name)}/kerberos/${id}`)
    }
  }

  const handleBack = () => {
    navigate(USER_FEDERATION_OVERVIEW_URL(realm_name))
  }

  const handleTestConnection = async () => {
    if (!realm_name || !id) return

    try {
      const result = await testConnection({
        path: { realm_name, id }
      })

      if (result.success) {
        toast.success('Connection test successful', {
          description: result.message
        })
      } else {
        toast.error('Connection test failed', {
          description: result.message
        })
      }
    } catch (error) {
      console.error('Failed to test connection', error)
      toast.error('Failed to test connection', {
        description: 'An unexpected error occurred'
      })
    }
  }

  const handleSyncUsers = async () => {
    if (!realm_name || !id) return

    try {
      const result = await syncUsers({
        path: { realm_name, id }
      })

      const stats = [
        result.created > 0 ? `${result.created} created` : null,
        result.updated > 0 ? `${result.updated} updated` : null,
        result.disabled > 0 ? `${result.disabled} disabled` : null,
        result.failed > 0 ? `${result.failed} failed` : null,
      ].filter(Boolean).join(', ')

      toast.success('User synchronization completed', {
        description: stats || `Processed ${result.total_processed} users`
      })
    } catch (error) {
      console.error('Failed to sync users', error)
      toast.error('Failed to sync users', {
        description: 'An unexpected error occurred during synchronization'
      })
    }
  }

  const handleSubmit = async (data: CreateLdapProviderSchema) => {
    if (!realm_name || !id) return

    try {
      // Parse URL to get host and port
      const url = new URL(data.connectionUrl.startsWith('ldap') ? data.connectionUrl : `ldap://${data.connectionUrl}`)
      const port = url.port ? parseInt(url.port) : (data.useTls ? 636 : 389)

      // Only encode password if it was changed (not empty)
      // If empty, keep the existing encrypted password from the config
      const encodedPassword = data.bindPassword
        ? btoa(data.bindPassword)
        : (config?.bind?.bind_password_encrypted || '')

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
            connection: {
              server_url: url.hostname,
              port: port,
              use_tls: data.useTls,
              use_starttls: false,
              connection_timeout_seconds: 30
            },
            bind: {
              bind_dn: data.bindDn,
              bind_password_encrypted: encodedPassword
            },
            search: {
              base_dn: data.baseDn,
              user_search_filter: data.userSearchFilter
            },
            attributes: {
              username: 'uid',
              email: 'mail',
              first_name: 'givenName',
              last_name: 'sn'
            }
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

  useEffect(() => {
    if (isTestingConnectionIsSuccess) {
      fire()
    }
  }, [isTestingConnectionIsSuccess, fire])

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)}>
        <LdapFormUi
          form={form}
          handleBack={handleBack}
          handleSubmit={form.handleSubmit(handleSubmit)}
          onTypeChange={handleTypeChange}
          onTestConnection={handleTestConnection}
          isTestingConnection={isTestingConnection}
          onSyncUsers={handleSyncUsers}
          isSyncingUsers={isSyncingUsers}
          isEditMode={true}
          hasChanges={hasChanges}
        />
      </form>
    </Form>
  )
}

export default function PageDetailLdapFeature() {
  const { realm_name, id } = useParams<RouterParams & { id: string }>()
  const { data: providerData, isLoading } = useGetUserFederation(realm_name || '', id || '')

  if (isLoading) {
    return <div className='p-4 text-center'>Loading provider...</div>
  }

  if (!providerData) {
    return <div className='p-4 text-center'>Provider not found</div>
  }

  return <LdapDetailForm providerData={providerData} realm_name={realm_name || ''} id={id || ''} />
}
