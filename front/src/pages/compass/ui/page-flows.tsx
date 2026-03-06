import { Schemas } from '@/api/api.client'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import { Heading } from '@/components/ui/heading'
import { Input } from '@/components/ui/input'
import { Compass, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { FlowList } from './flow-list'
import { StatsCards } from './stats-cards'

import CompassFlow = Schemas.CompassFlow
import FlowStats = Schemas.FlowStats

interface PageFlowsProps {
  flows: CompassFlow[]
  stats: FlowStats | null
  isLoading: boolean
  isError: boolean
  realmName?: string
}

type StatusFilter = 'all' | 'success' | 'failure' | 'pending'

const statusFilters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'success', label: 'Success' },
  { key: 'failure', label: 'Failure' },
  { key: 'pending', label: 'Pending' },
]

export default function PageFlows({ flows, stats, isLoading, isError, realmName }: PageFlowsProps) {
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [query, setQuery] = useState('')

  const filteredFlows = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()
    return flows.filter((flow) => {
      if (statusFilter !== 'all' && flow.status !== statusFilter) return false
      if (!normalizedQuery) return true
      const haystack = [flow.grant_type, flow.client_id, flow.user_id, flow.ip_address, flow.user_agent]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      return haystack.includes(normalizedQuery)
    })
  }, [flows, statusFilter, query])

  return (
    <div className='min-h-screen bg-gradient-to-b from-background via-background to-muted/40'>
      <div className='container mx-auto flex flex-col gap-6 p-6 md:p-10 max-w-none'>
        <div className='flex items-center gap-4'>
          <div className='h-10 w-10 rounded-2xl bg-primary/10 flex items-center justify-center'>
            <Compass className='size-5 text-primary' />
          </div>
          <div>
            <Heading size={3} weight='medium'>Compass</Heading>
            <div className='flex flex-wrap items-center gap-2 text-sm text-muted-foreground'>
              <span>Authentication flow traces & analytics</span>
              {realmName && (
                <Badge variant='outline' className='border-primary/30 text-primary'>
                  Realm: {realmName}
                </Badge>
              )}
            </div>
          </div>
        </div>

        {isError && (
          <Alert variant='destructive'>
            <Compass className='h-4 w-4' />
            <AlertTitle>Flows unavailable</AlertTitle>
            <AlertDescription>
              We couldn&apos;t fetch the latest flows. Please try again later.
            </AlertDescription>
          </Alert>
        )}

        <StatsCards stats={stats} isLoading={isLoading} />

        <div>
          <div className='flex items-center justify-between mb-3'>
            <h2 className='text-base font-semibold'>
              Authentication Flows ({filteredFlows.length})
            </h2>
            <div className='flex items-center gap-2'>
              <div className='relative'>
                <Search className='absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
                <Input
                  type='search'
                  placeholder='Search by client, user, grant type...'
                  className='pl-9 h-9 w-64 bg-background text-sm'
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                />
              </div>
            </div>
          </div>

          <div className='flex items-center gap-2 mb-4'>
            {statusFilters.map((f) => (
              <button
                key={f.key}
                onClick={() => setStatusFilter(f.key)}
                className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${statusFilter === f.key
                  ? 'bg-primary/10 text-primary border-primary/40'
                  : 'bg-transparent text-foreground border-border hover:bg-muted'
                  }`}
              >
                {f.label}
              </button>
            ))}
          </div>

          <FlowList flows={filteredFlows} isLoading={isLoading} />
        </div>
      </div>
    </div>
  )
}
