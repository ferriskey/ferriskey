import { useDeleteClient, useGetClient, useUpdateClient } from '@/api/client.api'
import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { Skeleton } from '@/components/ui/skeleton'
import PageClientMaintenanceFeature from '@/pages/client/feature/page-client-maintenance-feature'
import { RouterParams } from '@/routes/router'
import { APPLICATIONS_URL } from '@/routes/sub-router/applications.router'
import { Globe, Palette, Puzzle } from 'lucide-react'
import { useState } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import ApplicationDetailShell, { AppTab, TabDef } from '../ui/application-detail-shell'
import ApiAccessTab from '../ui/tabs/api-access-tab'
import ComingSoonTab from '../ui/tabs/coming-soon-tab'
import CredentialsTab from '../ui/tabs/credentials-tab'
import QuickstartTab from '../ui/tabs/quickstart-tab'
import SettingsTab, { ApplicationSettingsValues } from '../ui/tabs/settings-tab'
import { inferApplicationType } from '../types'

export default function PageApplicationDetailFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const realm = realm_name ?? 'master'

  const { data: clientResponse, isLoading, refetch } = useGetClient({
    realm,
    clientId: client_id ?? '',
  })

  const { mutateAsync: updateClient, isPending: isSaving } = useUpdateClient()
  const { mutateAsync: deleteClient } = useDeleteClient()
  const { mutateAsync: createRedirectUri } = useCreateRedirectUri()
  const { mutateAsync: deleteRedirectUri } = useDeleteRedirectUri()
  const [uriPending, setUriPending] = useState(false)

  const client = clientResponse?.data

  function handleBack() {
    navigate(APPLICATIONS_URL(realm))
  }

  async function handleSave(values: ApplicationSettingsValues) {
    if (!client) return
    await updateClient({
      body: {
        client_id: client.client_id,
        name: values.name,
        enabled: values.enabled,
        direct_access_grants_enabled: values.directAccessGrantsEnabled,
        oauth_device_code_grant_enabled: values.oauthDeviceCodeGrantEnabled,
        access_token_lifetime: values.accessTokenLifetime,
        refresh_token_lifetime: values.refreshTokenLifetime,
        id_token_lifetime: values.idTokenLifetime,
        temporary_token_lifetime: values.temporaryTokenLifetime,
      },
      path: { client_id: client.id, realm_name: realm },
    })
  }

  async function handleDelete() {
    if (!client) return
    try {
      await deleteClient({ path: { client_id: client.id, realm_name: realm } })
      toast.success('Application deleted')
      navigate(APPLICATIONS_URL(realm))
    } catch {
      // error surfaced by the mutation hook
    }
  }

  async function handleAddRedirectUri(value: string) {
    if (!client) return
    setUriPending(true)
    try {
      await createRedirectUri({ realmName: realm, clientId: client.id, payload: { value } })
      await refetch()
      toast.success('Redirect URI added')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to add redirect URI')
    } finally {
      setUriPending(false)
    }
  }

  async function handleDeleteRedirectUri(redirectUriId: string) {
    if (!client) return
    setUriPending(true)
    try {
      await deleteRedirectUri({ realmName: realm, clientId: client.id, redirectUriId })
      await refetch()
      toast.success('Redirect URI removed')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to remove redirect URI')
    } finally {
      setUriPending(false)
    }
  }

  if (isLoading || !client) {
    return (
      <div className='flex flex-col gap-6 p-8 md:p-12 max-w-3xl'>
        <Skeleton className='h-6 w-48' />
        <Skeleton className='h-40 w-full rounded-md' />
        <Skeleton className='h-40 w-full rounded-md' />
      </div>
    )
  }

  const appType = inferApplicationType(client)
  // Interactive (browser) clients can have IdP connections, branding and addons.
  // Machine-to-machine and device clients have no human login surface for these.
  const isInteractive = appType !== 'm2m' && appType !== 'device'

  const tabs: TabDef[] = [
    { id: 'quickstart', label: 'Quickstart' },
    { id: 'settings', label: 'Settings' },
    { id: 'credentials', label: 'Credentials' },
    ...(isInteractive ? [{ id: 'connections', label: 'Connections', soon: true } as TabDef] : []),
    { id: 'api-access', label: 'API Access' },
    ...(isInteractive ? [{ id: 'addons', label: 'Addons', soon: true } as TabDef] : []),
    ...(isInteractive
      ? [{ id: 'login-experience', label: 'Login Experience', soon: true } as TabDef]
      : []),
    { id: 'maintenance', label: 'Maintenance' },
  ]

  const requested = searchParams.get('tab') as AppTab | null
  function isSelectable(t: AppTab | null): t is AppTab {
    return !!t && tabs.some((tab) => tab.id === t && !tab.soon)
  }
  const activeTab: AppTab = isSelectable(requested) ? requested : 'quickstart'

  function handleSelectTab(tab: AppTab) {
    setSearchParams(
      (prev) => {
        const next = new URLSearchParams(prev)
        next.set('tab', tab)
        return next
      },
      { replace: true },
    )
  }

  // Arrow function on purpose: it relies on `client` being narrowed to non-null
  // by the guard above. A hoisted `function` declaration would lose that narrowing.
  const renderTab = () => {
    switch (activeTab) {
      case 'quickstart':
        return <QuickstartTab client={client} realm={realm} />
      case 'settings':
        return (
          <SettingsTab
            client={client}
            isSaving={isSaving}
            uriPending={uriPending}
            onSave={handleSave}
            onDelete={handleDelete}
            onAddRedirectUri={handleAddRedirectUri}
            onDeleteRedirectUri={handleDeleteRedirectUri}
          />
        )
      case 'credentials':
        return <CredentialsTab client={client} />
      case 'api-access':
        return <ApiAccessTab realm={realm} clientId={client.id} />
      case 'maintenance':
        return <PageClientMaintenanceFeature />
      case 'connections':
        return (
          <ComingSoonTab
            icon={Globe}
            title='Connections'
            description='Choose which identity providers and login methods this application can use.'
            points={[
              'Enable or disable realm identity providers per application',
              'Restrict social / enterprise connections to specific apps',
            ]}
          />
        )
      case 'login-experience':
        return (
          <ComingSoonTab
            icon={Palette}
            title='Login Experience'
            description='Customize the sign-in page branding shown for this application.'
            points={[
              'Per-application logo, colors and theme overrides',
              'Currently branding is configured at the realm level',
            ]}
          />
        )
      case 'addons':
        return (
          <ComingSoonTab
            icon={Puzzle}
            title='Addons'
            description='Enable protocol addons such as SAML and WS-Federation for this application.'
            points={['SAML 2.0 service provider', 'WS-Federation', 'Token customization addons']}
          />
        )
      default:
        return null
    }
  }

  return (
    <ApplicationDetailShell
      key={client.id}
      client={client}
      tabs={tabs}
      activeTab={activeTab}
      onSelectTab={handleSelectTab}
      onBack={handleBack}
    >
      {renderTab()}
    </ApplicationDetailShell>
  )
}
