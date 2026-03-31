import { useGetOrganization } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import {
  ORGANIZATION_ATTRIBUTES_URL,
  ORGANIZATION_SETTINGS_URL,
  ORGANIZATION_URL,
  ORGANIZATIONS_URL,
} from '@/routes/sub-router/organization.router'
import { ArrowLeft } from 'lucide-react'
import { Outlet, useLocation, useNavigate, useParams } from 'react-router'

export default function OrganizationLayout() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()
  const { pathname } = useLocation()
  const navigate = useNavigate()

  const { data: orgResponse } = useGetOrganization({
    realm: realm_name,
    organizationId,
  })

  const baseUrl = ORGANIZATION_URL(realm_name, organizationId)

  const tabs = [
    { key: 'settings', label: 'Settings', path: `${baseUrl}${ORGANIZATION_SETTINGS_URL}` },
    { key: 'attributes', label: 'Attributes', path: `${baseUrl}${ORGANIZATION_ATTRIBUTES_URL}` },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <button
            onClick={() => navigate(`${ORGANIZATIONS_URL(realm_name)}/overview`)}
            className='flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2'
          >
            <ArrowLeft className='h-3.5 w-3.5' />
            Organizations
          </button>
          <h1 className='text-2xl font-bold tracking-tight'>
            {orgResponse?.name ?? orgResponse?.alias ?? '—'}
          </h1>
          <p className='text-sm text-muted-foreground mt-1'>
            {orgResponse?.alias && (
              <span className='font-mono'>{orgResponse.alias}</span>
            )}
            {orgResponse?.domain && (
              <span className='ml-2 text-muted-foreground'>· {orgResponse.domain}</span>
            )}
          </p>
        </div>
        <div className='flex items-center gap-2 shrink-0'>
          <span
            className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${
              orgResponse?.enabled
                ? 'border-green-300 text-green-600 bg-green-50 dark:bg-green-500/10 dark:border-green-400/40'
                : 'border-border text-muted-foreground bg-muted/50'
            }`}
          >
            {orgResponse?.enabled ? 'enabled' : 'disabled'}
          </span>
        </div>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => navigate(tab.path)}
            className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
              pathname.startsWith(tab.path)
                ? 'bg-primary/10 text-primary border-primary/40'
                : 'bg-transparent text-foreground border-border hover:bg-muted'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <Outlet />
    </div>
  )
}
