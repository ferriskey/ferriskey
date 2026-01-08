import { useNavigate, useParams } from 'react-router'
import PageOverview from '../ui/page-overview'
import { RouterParams } from '@/routes/router'
import { USER_FEDERATION_URL, USER_FEDERATION_CREATE_URL } from '@/routes/sub-router/user-federation.router'
import { useGetUserFederations } from '@/api/user-federation.api'
import { Schemas } from '@/api/api.client'

export default function PageOverviewFeature() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { data: providersData, isLoading } = useGetUserFederations(realm_name || '')

  const handleCreateProvider = (type?: 'LDAP' | 'Kerberos') => {
    const url = `${USER_FEDERATION_URL(realm_name)}/${type ? type.toLowerCase() : ''}${USER_FEDERATION_CREATE_URL}/`
    navigate(url)
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
      providers={mappedProviders}
      isLoading={isLoading}
    />
  )
}
