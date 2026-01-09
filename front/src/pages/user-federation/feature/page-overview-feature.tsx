import { useNavigate, useParams } from 'react-router'
import PageOverview from '../ui/page-overview'
import { RouterParams } from '@/routes/router'
import { USER_FEDERATION_URL, USER_FEDERATION_CREATE_URL, USER_FEDERATION_LDAP_DETAIL_URL, USER_FEDERATION_KERBEROS_DETAIL_URL } from '@/routes/sub-router/user-federation.router'
import { useGetUserFederations, useDeleteUserFederation } from '@/api/user-federation.api'
import { Schemas } from '@/api/api.client'
import { toast } from 'sonner'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'

export default function PageOverviewFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { data: providersData, isLoading } = useGetUserFederations(realm_name || '')
  const { mutateAsync: deleteProvider } = useDeleteUserFederation()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const handleCreateProvider = (type?: 'LDAP' | 'Kerberos') => {
    const url = `${USER_FEDERATION_URL(realm_name)}/${type ? type.toLowerCase() : ''}${USER_FEDERATION_CREATE_URL}/`
    navigate(url)
  }

  const handleViewProvider = (id: string, type: string) => {
    if (!realm_name) return

    if (type === 'Ldap' || type === 'LDAP') {
      navigate(USER_FEDERATION_LDAP_DETAIL_URL(realm_name, id))
    } else if (type === 'Kerberos') {
      navigate(USER_FEDERATION_KERBEROS_DETAIL_URL(realm_name, id))
    }
  }

  const handleDeleteProvider = (id: string) => {
    if (!realm_name) return

    const provider = providersData?.data.find((p: Schemas.ProviderResponse) => p.id === id)
    if (!provider) return

    ask({
      title: 'Delete Provider?',
      description: `Are you sure you want to delete "${provider.name}"? This action cannot be undone.`,
      onConfirm: async () => {
        try {
          await deleteProvider({
            path: { realm_name, id }
          })
          toast.success('Provider deleted successfully')
          close()
        } catch (error) {
          console.error('Failed to delete provider', error)
          toast.error('Failed to delete provider')
        }
      },
    })
  }

  const mapPriority = (p: number) => {
    if (p === 0) return 'Primary'
    if (p === 10) return 'Secondary'
    if (p === 20) return 'Development'
    return 'Custom'
  }

  const mappedProviders = (providersData?.data ?? []).map((p: Schemas.ProviderResponse) => {
    const config = p.config as Record<string, unknown>
    return {
      id: p.id,
      name: p.name,
      type: typeof p.provider_type === 'string' ? p.provider_type : 'Custom',
      status: p.enabled ? 'active' : 'inactive' as 'active' | 'syncing' | 'inactive',
      users: 0, // Placeholder as API doesn't return user count yet
      lastSync: p.updated_at ? new Date(p.updated_at).toLocaleDateString() : 'Never',
      connection: (config?.connectionUrl as string) || (config?.kdcServer as string) || 'Unknown',
      priority: mapPriority(p.priority),
    }
  })

  return (
    <PageOverview
      onCreateProvider={handleCreateProvider}
      onDeleteProvider={handleDeleteProvider}
      onViewProvider={handleViewProvider}
      providers={mappedProviders}
      isLoading={isLoading}
      confirm={confirm}
      onConfirmClose={close}
    />
  )
}
