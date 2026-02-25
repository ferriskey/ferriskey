import { useGetRole } from '@/api/role.api'
import { RouterParams } from '@/routes/router'
import { ROLE_OVERVIEW_URL, ROLE_URL, ROLES_URL } from '@/routes/sub-router/role.router'
import { ArrowLeft } from 'lucide-react'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router-dom'

export default function RoleLayout() {
  const { realm_name, role_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { pathname } = useLocation()

  const { data: roleResponse } = useGetRole({
    realm: realm_name || 'master',
    roleId: role_id,
  })

  const baseUrl = ROLE_URL(realm_name, role_id)

  const tabs = [
    { key: 'settings', label: 'Settings', path: `${baseUrl}/settings` },
    { key: 'permissions', label: 'Permissions', path: `${baseUrl}/permissions` },
    { key: 'users', label: 'Users in role', path: `${baseUrl}/users` },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={() => navigate(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Roles
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>{roleResponse?.data.name}</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            {roleResponse?.data.description || `role_id: ${roleResponse?.data.id}`}
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          {roleResponse?.data.client_id ? (
            <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
              client
            </span>
          ) : (
            <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
              realm
            </span>
          )}
          <span className='inline-flex items-center px-3 py-0.5 rounded-md text-xs font-semibold border border-border text-muted-foreground bg-muted/50'>
            {roleResponse?.data.permissions?.length ?? 0} permission{(roleResponse?.data.permissions?.length ?? 0) !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
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

      <Outlet />
    </div>
  )
}
