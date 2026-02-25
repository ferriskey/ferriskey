import { useGetClient } from '@/api/client.api'
import { RouterParams } from '@/routes/router'
import { CLIENT_URL, CLIENTS_URL } from '@/routes/sub-router/client.router'
import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { Outlet } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'

export default function ClientLayout() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { pathname } = useLocation()
  const navigate = useNavigate()

  const { data: responseClient } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id,
  })

  const baseUrl = CLIENT_URL(realm_name, client_id)

  const tabs = [
    { key: 'settings', label: 'Settings', path: `${baseUrl}/settings` },
    ...(responseClient?.data.secret
      ? [{ key: 'credentials', label: 'Credentials', path: `${baseUrl}/credentials` }]
      : []),
    { key: 'roles', label: 'Roles', path: `${baseUrl}/roles` },
    { key: 'client-scopes', label: 'Client Scopes', path: '', disabled: true },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={() => navigate(`${CLIENTS_URL(realm_name)}/overview`)}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Clients
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>{responseClient?.data.client_id}</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Clients are applications and services that can request authentication of a user.
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${
              responseClient?.data.enabled
                ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                : 'border-border text-muted-foreground bg-muted/50'
            }`}
          >
            {responseClient?.data.enabled ? 'enabled' : 'disabled'}
          </span>
          <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-primary/40 text-primary text-xs font-mono bg-primary/10'>
            {responseClient?.data.protocol}
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
        {tabs.map((tab) => {
          const isActive = pathname.startsWith(tab.path) && tab.path !== ''
          const isDisabled = 'disabled' in tab && tab.disabled

          return (
            <button
              key={tab.key}
              onClick={() => !isDisabled && tab.path && navigate(tab.path)}
              disabled={isDisabled}
              className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
                isDisabled
                  ? 'bg-transparent text-muted-foreground border-border opacity-50 cursor-not-allowed'
                  : isActive
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
