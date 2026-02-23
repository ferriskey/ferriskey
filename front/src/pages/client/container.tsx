import PageContainer from '@/components/ui/page-container'
import { OverviewHeader } from '@/components/ui/overview-header'
import { useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { CLIENTS_URL, CLIENT_CREATE_URL } from '@/routes/sub-router/client.router'
import { USERS_URL } from '@/routes/sub-router/user.router'
import { ROLES_URL } from '@/routes/sub-router/role.router'
import { useLocation } from 'react-router'

export default function Container() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  const tabs = [
    {
      key: 'clients',
      label: 'Clients',
      onClick: () => navigate(`${CLIENTS_URL(realm_name)}/overview`),
      active: location.pathname.startsWith(CLIENTS_URL(realm_name)),
    },
    {
      key: 'users',
      label: 'Users',
      onClick: () => navigate(`${USERS_URL(realm_name)}/overview`),
      active: location.pathname.startsWith(USERS_URL(realm_name)),
    },
    {
      key: 'roles',
      label: 'Roles',
      onClick: () => navigate(`${ROLES_URL(realm_name)}/overview`),
      active: location.pathname.startsWith(ROLES_URL(realm_name)),
    },
    {
      key: 'client-scopes',
      label: 'Client Scopes',
      onClick: () => {},
      active: false,
    },
  ]

  return (
    <PageContainer>
      <OverviewHeader
        title='Client and Access Administration'
        description='Manage realm settings, users, and policy workflows'
        primaryAction={{
          label: 'New Client',
          onClick: () => navigate(`${CLIENTS_URL(realm_name)}${CLIENT_CREATE_URL}`),
        }}
        secondaryAction={{
          label: 'Import',
          onClick: () => {},
        }}
        tabs={tabs}
      />
    </PageContainer>
  )
}
