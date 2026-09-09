import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs, type TabItem } from '@/components/kit'
import { NEXT_CLIENTS_URL } from '@/next/routes'
import { NEXT_CLIENT_URL } from '../client-routes'
import PageClientDetail from '../ui/page-client-detail'
import ClientSettingsTabFeature from './client-settings-tab-feature'
import ClientCredentialsTabFeature from './client-credentials-tab-feature'
import ClientRolesTabFeature from './client-roles-tab-feature'
import ClientScopesTabFeature from './client-scopes-tab-feature'
import ClientSamlTabFeature from './client-saml-tab-feature'
import ClientMaintenanceTabFeature from './client-maintenance-tab-feature'
import { useCrumbLabel } from '@/next/shell/crumb-store'

export default function PageClientDetailFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: clientResponse, isLoading } = useGetClient({ realm, clientId: client_id })
  const client = clientResponse?.data

  const hasSecret = Boolean(client?.secret)
  const isSaml = client?.protocol === 'saml'

  const tabList = useMemo<TabItem[]>(
    () => [
      { key: 'settings', label: 'Settings' },
      ...(hasSecret ? [{ key: 'credentials', label: 'Credentials' }] : []),
      { key: 'roles', label: 'Roles' },
      { key: 'scopes', label: 'Client Scopes' },
      ...(isSaml ? [{ key: 'saml', label: 'SAML' }] : []),
      { key: 'maintenance', label: 'Maintenance' },
    ],
    [hasSecret, isSaml]
  )

  const { value: tab, tabs } = useRouteTabs(NEXT_CLIENT_URL(realm, client_id), tabList)


  useCrumbLabel(client_id, client?.name ?? client?.client_id)

  return (
    <PageClientDetail
      client={client}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      onBack={() => navigate(NEXT_CLIENTS_URL(realm))}
    >
      {client && tab === 'settings' && <ClientSettingsTabFeature client={client} realm={realm} />}
      {client && tab === 'credentials' && (
        <ClientCredentialsTabFeature client={client} realm={realm} />
      )}
      {client && tab === 'roles' && <ClientRolesTabFeature client={client} realm={realm} />}
      {client && tab === 'scopes' && <ClientScopesTabFeature client={client} realm={realm} />}
      {client && isSaml && tab === 'saml' && (
        <ClientSamlTabFeature client={client} realm={realm} />
      )}
      {client && tab === 'maintenance' && (
        <ClientMaintenanceTabFeature client={client} realm={realm} />
      )}
    </PageClientDetail>
  )
}
