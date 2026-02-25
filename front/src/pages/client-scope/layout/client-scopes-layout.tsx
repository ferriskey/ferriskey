import { OverviewHeader } from '@/components/ui/overview-header'
import { RouterParams } from '@/routes/router'
import { CLIENTS_URL } from '@/routes/sub-router/client.router'
import {
  CLIENT_SCOPES_CREATE_URL,
  CLIENT_SCOPES_OVERVIEW_URL,
  CLIENT_SCOPES_URL,
} from '@/routes/sub-router/client-scope.router'
import { ROLES_URL } from '@/routes/sub-router/role.router'
import { USERS_URL } from '@/routes/sub-router/user.router'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router'

export default function ClientScopesLayout() {
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
      onClick: () => navigate(`${CLIENT_SCOPES_URL(realm_name)}${CLIENT_SCOPES_OVERVIEW_URL}`),
      active: location.pathname.startsWith(CLIENT_SCOPES_URL(realm_name)),
    },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      <OverviewHeader
        title='Client and Access Administration'
        description='Manage realm settings, users, and policy workflows'
        primaryAction={{
          label: 'New Client Scope',
          onClick: () => navigate(`${CLIENT_SCOPES_URL(realm_name)}${CLIENT_SCOPES_CREATE_URL}`),
        }}
        tabs={tabs}
      />
      <Outlet />
    </div>
  )
}
