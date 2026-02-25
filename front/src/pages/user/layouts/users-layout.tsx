import { OverviewHeader } from '@/components/ui/overview-header'
import { useNavigate, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { USERS_URL, USER_CREATE_URL } from '@/routes/sub-router/user.router'
import { CLIENTS_URL } from '@/routes/sub-router/client.router'
import { ROLES_URL } from '@/routes/sub-router/role.router'
import { useLocation, Outlet } from 'react-router-dom'

export default function UsersLayout() {
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
    <div className='flex flex-col gap-6 p-8'>
      <OverviewHeader
        title='Client and Access Administration'
        description='Manage realm settings, users, and policy workflows'
        primaryAction={{
          label: 'New User',
          onClick: () => navigate(`${USERS_URL(realm_name)}${USER_CREATE_URL}`),
        }}
        tabs={tabs}
      />
      <Outlet />
    </div>
  )
}
