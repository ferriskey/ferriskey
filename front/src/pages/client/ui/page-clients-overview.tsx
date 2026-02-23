import { Schemas } from '@/api/api.client.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import StatisticsCard from '../components/statistics-card'
import ClientTrendChart from '../components/client-trend-chart'
import { useState, useMemo } from 'react'
import { AlertTriangle, Search, SlidersHorizontal } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'

import Client = Schemas.Client

type QuickFilter = 'all' | 'confidential' | 'deprecated'

interface Statistics {
  totalClients: number
  activeClients: number
  publicClients: number
  confidentialClients: number
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

export interface PageClientsOverviewProps {
  isLoading?: boolean
  data: Client[]
  realmName: string
  statistics: Statistics
  filters: Filter[]
  filterFields: FilterFieldsConfig
  onFiltersChange: (filters: Filter[]) => void
  confirm: ConfirmState
  onConfirmClose: () => void
  handleDeleteSelected: (items: Client[]) => void
  handleClickRow: (clientId: string) => void
  handleCreateClient: () => void
  onRowDelete: (client: Client) => void
}

const quickFilters: { key: QuickFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'confidential', label: 'Confidential' },
  { key: 'deprecated', label: 'Deprecated' },
]

const PAGE_SIZE = 10

function ClientAvatar({ name }: { name: string }) {
  return (
    <div className='h-10 w-10 rounded-md flex items-center justify-center shrink-0' style={{ backgroundColor: '#F97316' }}>
      <span className='text-base font-bold text-white'>{name?.[0]?.toUpperCase() || 'C'}</span>
    </div>
  )
}

function StatusBadge({ enabled }: { enabled: boolean }) {
  if (!enabled) {
    return (
      <span className='inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold border border-orange-400/50 text-orange-500 bg-orange-50 dark:bg-orange-500/10'>
        <AlertTriangle className='h-3 w-3' />
        INACTIVE
      </span>
    )
  }
  return (
    <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10'>
      ACTIVE
    </span>
  )
}

function TypeBadge({ isPublic }: { isPublic: boolean }) {
  return isPublic ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
      public
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
      confidential
    </span>
  )
}

export default function PageClientsOverview({
  data,
  isLoading,
  statistics,
  confirm,
  onConfirmClose,
  handleClickRow,
}: PageClientsOverviewProps) {
  const { totalClients, activeClients, publicClients, confidentialClients } = statistics
  const [quickFilter, setQuickFilter] = useState<QuickFilter>('all')
  const [search, setSearch] = useState('')
  const [currentPage, setCurrentPage] = useState(1)

  const filteredData = useMemo(() => {
    let result = data

    if (quickFilter === 'confidential') result = result.filter((c) => !c.public_client)
    else if (quickFilter === 'deprecated') result = result.filter((c) => !c.enabled)

    if (search.trim()) {
      const q = search.toLowerCase()
      result = result.filter(
        (c) => c.name?.toLowerCase().includes(q) || c.client_id?.toLowerCase().includes(q)
      )
    }

    return result
  }, [data, quickFilter, search])

  const totalPages = Math.ceil(filteredData.length / PAGE_SIZE)
  const paginatedData = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE
    return filteredData.slice(start, start + PAGE_SIZE)
  }, [filteredData, currentPage])

  const handleQuickFilter = (f: QuickFilter) => {
    setQuickFilter(f)
    setCurrentPage(1)
  }

  const rangeStart = filteredData.length === 0 ? 0 : (currentPage - 1) * PAGE_SIZE + 1
  const rangeEnd = Math.min(currentPage * PAGE_SIZE, filteredData.length)

  return (
    <div className='flex flex-col gap-6'>
      {/* Quick Filters */}
      <div className='flex items-center gap-2'>
        {quickFilters.map((f) => (
          <button
            key={f.key}
            onClick={() => handleQuickFilter(f.key)}
            className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${quickFilter === f.key
              ? 'bg-primary/10 text-primary border-primary/40'
              : 'bg-transparent text-foreground border-border hover:bg-muted'
              }`}
          >
            {f.label}
          </button>
        ))}
      </div>

      {/* Statistics Cards */}
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Client overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Total clients'
            value={totalClients}
            isLoading={isLoading}
            chart={!isLoading && <ClientTrendChart clients={data} days={7} color='hsl(200 76% 36%)' />}
          />
          <StatisticsCard
            title='Active clients'
            value={activeClients}
            isLoading={isLoading}
            chart={
              !isLoading && (
                <ClientTrendChart clients={data.filter((c) => c.enabled)} days={7} color='hsl(142 76% 36%)' />
              )
            }
          />
          <StatisticsCard
            title='Public clients'
            value={publicClients}
            isLoading={isLoading}
            chart={
              !isLoading && (
                <ClientTrendChart clients={data.filter((c) => c.public_client)} days={7} color='hsl(30 100% 50%)' />
              )
            }
          />
          <StatisticsCard
            title='Confidential'
            value={confidentialClients}
            isLoading={isLoading}
            chart={
              !isLoading && (
                <ClientTrendChart clients={data.filter((c) => !c.public_client)} days={7} color='hsl(262 83% 58%)' />
              )
            }
          />
        </div>
      </div>

      {/* Client List */}
      <div>
        {/* List header */}
        <div className='flex items-center justify-between mb-3'>
          <h2 className='text-base font-semibold'>
            Clients ({filteredData.length})
          </h2>
          <div className='flex items-center gap-2'>
            <div className='relative'>
              <Search className='absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
              <Input
                type='search'
                placeholder='Search clients...'
                className='pl-9 h-9 w-64 bg-background text-sm'
                value={search}
                onChange={(e) => {
                  setSearch(e.target.value)
                  setCurrentPage(1)
                }}
              />
            </div>
            <Button variant='ghost' size='icon' className='h-9 w-9'>
              <SlidersHorizontal className='h-4 w-4' />
            </Button>
          </div>
        </div>

        {/* List body */}
        <div className='-mx-8 border-t border-b overflow-hidden'>
          {isLoading ? (
            Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className='flex items-center justify-between px-8 py-4 border-b last:border-b-0'>
                <div className='flex items-center gap-3'>
                  <Skeleton className='h-10 w-10 rounded-lg' />
                  <div className='space-y-2'>
                    <Skeleton className='h-4 w-40' />
                    <Skeleton className='h-3 w-32' />
                  </div>
                </div>
                <Skeleton className='h-6 w-16 rounded-md' />
              </div>
            ))
          ) : paginatedData.length === 0 ? (
            <div className='flex items-center justify-center h-24 text-sm text-muted-foreground'>
              No clients found.
            </div>
          ) : (
            paginatedData.map((client) => (
              <div
                key={client.id}
                onClick={() => handleClickRow(client.id)}
                className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 cursor-pointer transition-colors'
              >
                {/* Left: avatar + name + client_id + type badge */}
                <div className='flex items-center gap-4'>
                  <ClientAvatar name={client.name} />
                  <div>
                    <div className='flex items-center gap-2.5'>
                      <span className='text-base font-medium'>{client.name}</span>
                      <TypeBadge isPublic={client.public_client} />
                    </div>
                    <div className='text-sm text-muted-foreground mt-0.5'>
                      client_id: {client.client_id}
                    </div>
                  </div>
                </div>

                {/* Right: status badge */}
                <StatusBadge enabled={client.enabled} />
              </div>
            ))
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className='flex items-center justify-between mt-4 px-1'>
            <span className='text-sm text-muted-foreground'>
              {rangeStart}-{rangeEnd} sur {filteredData.length}
            </span>
            <div className='flex items-center gap-1'>
              <Button
                variant='outline'
                size='sm'
                onClick={() => setCurrentPage((p) => Math.max(p - 1, 1))}
                disabled={currentPage <= 1}
                className='h-8'
              >
                Precedent
              </Button>
              {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => (
                <Button
                  key={page}
                  variant={page === currentPage ? 'default' : 'outline'}
                  size='sm'
                  onClick={() => setCurrentPage(page)}
                  className='h-8 w-8 p-0'
                >
                  {page}
                </Button>
              ))}
              <Button
                variant='outline'
                size='sm'
                onClick={() => setCurrentPage((p) => Math.min(p + 1, totalPages))}
                disabled={currentPage >= totalPages}
                className='h-8'
              >
                Suivant
              </Button>
            </div>
          </div>
        )}
      </div>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={onConfirmClose}
      />
    </div>
  )
}
