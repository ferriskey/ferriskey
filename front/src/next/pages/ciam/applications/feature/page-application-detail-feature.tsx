import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs, type TabItem } from '@/components/kit'
import ClientCredentialsTabFeature from '@/next/pages/iam/client/feature/client-credentials-tab-feature'
import ClientScopesTabFeature from '@/next/pages/iam/client/feature/client-scopes-tab-feature'
import ClientSamlTabFeature from '@/next/pages/iam/client/feature/client-saml-tab-feature'
import ClientMaintenanceTabFeature from '@/next/pages/iam/client/feature/client-maintenance-tab-feature'
import { CONSOLE_APPLICATIONS_URL, CONSOLE_APPLICATION_URL } from '../application-routes'
import { applicationTypeMeta, inferApplicationType } from '../application-types'
import PageApplicationDetail from '../ui/page-application-detail'
import ApplicationCredentialsNote from '../ui/application-credentials-note'
import ApplicationQuickstartTabFeature from './application-quickstart-tab-feature'
import ApplicationSettingsTabFeature from './application-settings-tab-feature'

export default function PageApplicationDetailFeature() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: clientResponse, isLoading } = useGetClient({ realm, clientId: client_id })
  const application = clientResponse?.data

  const type = application ? inferApplicationType(application) : null
  const isInteractive = type ? applicationTypeMeta(type).usesAuthorizationCode : false

  const tabList = useMemo<TabItem[]>(
    () => [
      { key: 'quickstart', label: 'Quickstart' },
      { key: 'settings', label: 'Settings' },
      { key: 'credentials', label: 'Credentials' },
      { key: 'api-access', label: 'API access' },
      ...(isInteractive ? [{ key: 'saml', label: 'SAML' }] : []),
      { key: 'maintenance', label: 'Maintenance' },
    ],
    [isInteractive]
  )

  const { value: tab, tabs } = useRouteTabs(CONSOLE_APPLICATION_URL(realm, client_id), tabList)

  return (
    <PageApplicationDetail
      application={application}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      onBack={() => navigate(CONSOLE_APPLICATIONS_URL(realm))}
    >
      {application && tab === 'quickstart' && (
        <ApplicationQuickstartTabFeature application={application} realm={realm} />
      )}
      {application && tab === 'settings' && (
        <ApplicationSettingsTabFeature application={application} realm={realm} />
      )}
      {application && tab === 'credentials' && type && (
        application.secret ? (
          <ClientCredentialsTabFeature client={application} realm={realm} />
        ) : (
          <ApplicationCredentialsNote type={type} clientId={application.client_id} />
        )
      )}
      {application && tab === 'api-access' && (
        <ClientScopesTabFeature client={application} realm={realm} />
      )}
      {application && isInteractive && tab === 'saml' && (
        <ClientSamlTabFeature client={application} realm={realm} />
      )}
      {application && tab === 'maintenance' && (
        <ClientMaintenanceTabFeature client={application} realm={realm} />
      )}
    </PageApplicationDetail>
  )
}
