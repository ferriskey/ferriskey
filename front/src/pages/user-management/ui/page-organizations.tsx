import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { Building2, ChevronDown, CircleDot, Globe, Plus, Search, ShieldOff } from 'lucide-react'
import { useMemo, useState } from 'react'

import Organization = Schemas.Organization

interface Props {
  organizations: Organization[]
  isLoading: boolean
  onSelect: (organizationId: string) => void
  onCreate: () => void
}

type StatusFilter = 'all' | 'enabled' | 'disabled'

const statusFilters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'enabled', label: 'Enabled' },
  { key: 'disabled', label: 'Disabled' },
]

type SortKey = 'recent' | 'name' | 'alias'
const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
  alias: 'Alias (A→Z)',
}

const formatRelative = (iso: string) => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return '—'
  const diffMs = Date.now() - date.getTime()
  const sec = Math.round(diffMs / 1000)
  if (sec < 60) return 'just now'
  const min = Math.round(sec / 60)
  if (min < 60) return `${min}m ago`
  const hr = Math.round(min / 60)
  if (hr < 24) return `${hr}h ago`
  const days = Math.round(hr / 24)
  if (days < 30) return `${days}d ago`
  return date.toLocaleDateString()
}

const initials = (name: string) => {
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) return `${parts[0][0]}${parts[1][0]}`.toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

export default function PageOrganizations({ organizations, isLoading, onSelect, onCreate }: Props) {
  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = organizations.length
    const enabled = organizations.filter((o) => o.enabled).length
    const withDomain = organizations.filter((o) => !!o.domain).length
    return { total, enabled, withDomain, disabled: total - enabled }
  }, [organizations])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = organizations.filter((o) => {
      if (statusFilter === 'enabled' && !o.enabled) return false
      if (statusFilter === 'disabled' && o.enabled) return false
      if (!q) return true
      const hay = [o.name, o.alias, o.domain, o.description].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })
    list = [...list]
    if (sort === 'recent') list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    if (sort === 'name') list.sort((a, b) => a.name.localeCompare(b.name))
    if (sort === 'alias') list.sort((a, b) => a.alias.localeCompare(b.alias))
    return list
  }, [organizations, query, statusFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-8 md:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Organizations</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Group identities into B2B tenants with their own domains and sign-in flows.
          </p>
        </div>
        <button
          onClick={onCreate}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          Create organization
        </button>
      </div>

      {/* Stats */}
      <div className='grid grid-cols-2 lg:grid-cols-3 gap-4'>
        <SmallStat icon={Building2} tone='emerald' label='Total organizations' value={stats.total} isLoading={isLoading} />
        <SmallStat icon={CircleDot} tone='blue' label='Enabled' value={stats.enabled} isLoading={isLoading} />
        <SmallStat icon={Globe} tone='amber' label='With custom domain' value={stats.withDomain} isLoading={isLoading} />
      </div>

      {/* Toolbar */}
      <div className='flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
        <div className='flex items-center gap-2 flex-wrap'>
          {statusFilters.map((f) => (
            <button
              key={f.key}
              onClick={() => setStatusFilter(f.key)}
              className={`px-3.5 py-1.5 rounded-md text-xs font-medium transition-colors border ${
                statusFilter === f.key
                  ? 'bg-primary/10 text-primary border-primary/40'
                  : 'bg-transparent text-foreground border-border hover:bg-muted'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
        <div className='flex items-center gap-2'>
          <div className='relative'>
            <Search className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search by name, alias, domain…'
              className='w-72 rounded-md border border-border bg-background pl-9 pr-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </div>
          <div className='relative'>
            <select
              value={sort}
              onChange={(e) => setSort(e.target.value as SortKey)}
              className='appearance-none rounded-md border border-border bg-background pl-3 pr-8 py-1.5 text-sm font-medium hover:bg-muted transition-colors cursor-pointer outline-none'
            >
              {(Object.keys(sortLabels) as SortKey[]).map((k) => (
                <option key={k} value={k}>
                  {sortLabels[k]}
                </option>
              ))}
            </select>
            <ChevronDown className='pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
          </div>
        </div>
      </div>

      {/* Grid */}
      {isLoading ? (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {Array.from({ length: 6 }).map((_, i) => (
            <div key={`org-skel-${i}`} className='rounded-md border border-border bg-card/40 p-5 space-y-3'>
              <div className='flex items-center gap-3'>
                <Skeleton className='h-10 w-10 rounded-md' />
                <div className='space-y-2 flex-1'>
                  <Skeleton className='h-3 w-32' />
                  <Skeleton className='h-2 w-20' />
                </div>
              </div>
              <Skeleton className='h-3 w-full' />
              <Skeleton className='h-3 w-2/3' />
            </div>
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className='rounded-md border border-dashed border-border bg-muted/20 p-12 flex flex-col items-center justify-center text-center gap-3'>
          <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
            <Building2 className='h-6 w-6 text-muted-foreground' />
          </div>
          <div>
            <p className='text-sm font-medium'>No organizations match your filters</p>
            <p className='text-xs text-muted-foreground mt-1'>
              {organizations.length === 0
                ? 'Create your first organization to start grouping identities.'
                : 'Adjust the search or status filter.'}
            </p>
          </div>
        </div>
      ) : (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {filtered.map((o) => (
            <button
              key={o.id}
              onClick={() => onSelect(o.id)}
              className='group flex flex-col gap-4 rounded-md border border-border bg-card/40 p-5 text-left transition hover:border-primary/30 hover:bg-muted/40 hover:shadow-sm'
            >
              <div className='flex items-start gap-3'>
                <div className='h-10 w-10 rounded-md bg-primary/10 text-primary flex items-center justify-center text-sm font-semibold shrink-0'>
                  {initials(o.name || o.alias)}
                </div>
                <div className='flex-1 min-w-0'>
                  <div className='flex items-center gap-2'>
                    <span className='text-sm font-medium truncate group-hover:text-primary transition-colors'>
                      {o.name}
                    </span>
                    {!o.enabled && (
                      <span className='inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                        <ShieldOff className='h-2.5 w-2.5' />
                        Off
                      </span>
                    )}
                  </div>
                  <p className='text-xs text-muted-foreground truncate'>@{o.alias}</p>
                </div>
              </div>

              {o.description && (
                <p className='text-xs text-muted-foreground line-clamp-2'>{o.description}</p>
              )}

              <div className='flex items-center gap-3 text-xs text-muted-foreground mt-auto pt-2 border-t border-border/60'>
                {o.domain ? (
                  <span className='inline-flex items-center gap-1 truncate'>
                    <Globe className='h-3 w-3' />
                    {o.domain}
                  </span>
                ) : (
                  <span className='inline-flex items-center gap-1 text-muted-foreground/60'>
                    <Globe className='h-3 w-3' />
                    No domain
                  </span>
                )}
                <span className='ml-auto whitespace-nowrap'>{formatRelative(o.created_at)}</span>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

interface SmallStatProps {
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue' | 'amber' | 'muted'
  label: string
  value: number
  isLoading: boolean
}

function SmallStat({ icon: Icon, tone, label, value, isLoading }: SmallStatProps) {
  const tones: Record<SmallStatProps['tone'], { bg: string; fg: string }> = {
    emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500' },
    blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500' },
    amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500' },
    muted: { bg: 'bg-muted', fg: 'text-muted-foreground' },
  }
  const t = tones[tone]
  return (
    <div className='flex items-center gap-3 rounded-md border border-border bg-card/40 px-4 py-3'>
      <div className={`h-9 w-9 rounded-md flex items-center justify-center ${t.bg}`}>
        <Icon className={`h-4 w-4 ${t.fg}`} />
      </div>
      <div className='min-w-0'>
        <div className='text-xl font-semibold tabular-nums leading-none'>
          {isLoading ? <Skeleton className='h-5 w-10' /> : value}
        </div>
        <p className='text-xs text-muted-foreground mt-1 truncate'>{label}</p>
      </div>
    </div>
  )
}
