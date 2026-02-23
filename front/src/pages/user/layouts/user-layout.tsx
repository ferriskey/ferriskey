import { Outlet, useLocation, useNavigate, useParams } from 'react-router'
import { useGetUser } from '../../../api/user.api'
import { UserRouterParams, USERS_URL } from '../../../routes/sub-router/user.router'
import { ArrowLeft } from 'lucide-react'
import RoleMappingModalFeature from '../feature/modals/role-mapping-modal-feature'

export default function UserLayout() {
  const navigate = useNavigate()
  const { realm_name, user_id } = useParams<UserRouterParams>()
  const { pathname } = useLocation()

  const { data: userResponse } = useGetUser({
    realm: realm_name,
    userId: user_id,
  })

  const baseUrl = `/realms/${realm_name}/users/${user_id}`
  const isRoleMappingTab = pathname.includes('role-mapping')

  const tabs = [
    { key: 'overview', label: 'Overview', path: `${baseUrl}/overview` },
    { key: 'credentials', label: 'Credentials', path: `${baseUrl}/credentials` },
    { key: 'role-mapping', label: 'Role Mapping', path: `${baseUrl}/role-mapping` },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={() => navigate(`${USERS_URL(realm_name)}/overview`)}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Users
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>{userResponse?.data.username}</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            {userResponse?.data.firstname} {userResponse?.data.lastname}
            {userResponse?.data.email && (
              <span className='ml-2 text-muted-foreground'>· {userResponse?.data.email}</span>
            )}
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${
              userResponse?.data.email_verified
                ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                : 'border-amber-300 text-amber-600 bg-amber-50 dark:bg-amber-500/10 dark:border-amber-400/40'
            }`}
          >
            {userResponse?.data.email_verified ? 'verified' : 'unverified'}
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center justify-between -mt-2'>
        <div className='flex items-center gap-2'>
          {tabs.map((tab) => {
            const isActive = pathname.startsWith(tab.path)
            return (
              <button
                key={tab.key}
                onClick={() => navigate(tab.path)}
                className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
                  isActive
                    ? 'bg-primary/10 text-primary border-primary/40'
                    : 'bg-transparent text-foreground border-border hover:bg-muted'
                }`}
              >
                {tab.label}
              </button>
            )
          })}
        </div>
        {isRoleMappingTab && <RoleMappingModalFeature />}
      </div>

      <Outlet />
    </div>
  )
}
