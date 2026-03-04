import { useGetClientScope } from '@/api/client-scope.api'
import { ArrowLeft } from 'lucide-react'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import {
  CLIENT_SCOPE_DETAILS_URL,
  CLIENT_SCOPE_MAPPERS_URL,
  CLIENT_SCOPE_URL,
  CLIENT_SCOPES_OVERVIEW_URL,
  CLIENT_SCOPES_URL,
} from '@/routes/sub-router/client-scope.router'
import { cn } from '@/lib/utils'

export default function ClientScopeLayout() {
  const { realm_name, scope_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const location = useLocation()

  const { data: responseScope } = useGetClientScope({
    realm: realm_name ?? 'master',
    scopeId: scope_id,
  })

  const scopeType = responseScope?.default_scope_type ?? 'NONE'
  const isDefault = scopeType === 'DEFAULT'
  const isOptional = scopeType === 'OPTIONAL'

  const scopeBase = CLIENT_SCOPE_URL(realm_name, scope_id)

  const tabs = [
    {
      key: 'details',
      label: 'Details',
      path: `${scopeBase}${CLIENT_SCOPE_DETAILS_URL}`,
      active: location.pathname.endsWith(CLIENT_SCOPE_DETAILS_URL),
    },
    {
      key: 'mappers',
      label: 'Protocol Mappers',
      path: `${scopeBase}${CLIENT_SCOPE_MAPPERS_URL}`,
      // active for both /mappers and /mappers/new
      active: location.pathname.startsWith(`${scopeBase}${CLIENT_SCOPE_MAPPERS_URL}`),
    },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={() => navigate(`${CLIENT_SCOPES_URL(realm_name)}${CLIENT_SCOPES_OVERVIEW_URL}`)}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Client Scopes
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>{responseScope?.name || 'Client Scope'}</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            {responseScope?.description || 'No description provided'}
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${responseScope?.default_scope_type === 'DEFAULT'
              ? 'border-blue-300 text-blue-500 bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'
              : 'border-purple-300 text-purple-500 bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'
              }`}
          >
            {responseScope?.default_scope_type === 'DEFAULT' ? 'default' : 'optional'}
          </span>
          <span className='inline-flex items-center px-3 py-0.5 rounded-md text-xs font-semibold border border-border text-muted-foreground bg-muted/50'>
            {responseScope?.protocol || 'openid-connect'}
          </span>
        </div>

        <nav className='flex gap-1'>
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => navigate(tab.path)}
              className={cn(
                'px-4 py-2 text-sm font-medium border-b-2 transition-colors',
                tab.active
                  ? 'border-foreground text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground hover:border-muted-foreground'
              )}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      <Outlet />
    </div>
  )
}
