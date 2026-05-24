import { Skeleton } from '@/components/ui/skeleton'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import {
  CheckCircle2,
  ChevronDown,
  Clock,
  Database,
  Key,
  Pencil,
  Plus,
  Search,
  Server,
  Trash2,
  Users,
  XCircle,
} from 'lucide-react'
import { useMemo, useState } from 'react'

interface Provider {
  id: string
  name: string
  type: string
  status: 'active' | 'syncing' | 'inactive'
  users: number
  lastSync: string
  connection: string
  priority: string
}

interface ConfirmState {
  title: string
  description: string
  open: boolean
  onConfirm: () => void
}

interface PageOverviewProps {
  onCreateProvider: (type?: 'LDAP' | 'Kerberos') => void
  onDeleteProvider: (id: string) => void
  onViewProvider: (id: string, type: string) => void
  providers?: Provider[]
  isLoading?: boolean
  confirm: ConfirmState
  onConfirmClose: () => void
}

type ProviderType = 'LDAP' | 'Kerberos'
type TypeFilter = 'all' | ProviderType
type StatusFilter = 'all' | 'active' | 'inactive'
type SortKey = 'recent' | 'name'

const PROVIDER_TYPES: { key: ProviderType; label: string; short: string; icon: typeof Server }[] = [
  { key: 'LDAP', label: 'LDAP Directory', short: 'LDAP', icon: Server },
  { key: 'Kerberos', label: 'Kerberos', short: 'Kerberos', icon: Key },
]

const PROVIDER_TONE: Record<ProviderType, { bg: string; fg: string; border: string }> = {
  LDAP: {
    bg: 'bg-blue-500/10',
    fg: 'text-blue-500',
    border: 'border-blue-500/40',
  },
  Kerberos: {
    bg: 'bg-amber-500/10',
    fg: 'text-amber-500',
    border: 'border-amber-500/40',
  },
}

const STATUS_TONE = {
  active: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500', border: 'border-emerald-500/40' },
  inactive: { bg: 'bg-zinc-500/10', fg: 'text-zinc-500', border: 'border-zinc-500/40' },
}

const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
}

const statusFilters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'active', label: 'Active' },
  { key: 'inactive', label: 'Inactive' },
]

const formatRelative = (dateStr: string) => {
  if (dateStr === 'Never') return dateStr
  const d = new Date(dateStr)
  if (Number.isNaN(d.getTime())) return dateStr
  const diffMs = Date.now() - d.getTime()
  const sec = Math.round(diffMs / 1000)
  if (sec < 60) return 'just now'
  const min = Math.round(sec / 60)
  if (min < 60) return `${min}m ago`
  const hr = Math.round(min / 60)
  if (hr < 24) return `${hr}h ago`
  const days = Math.round(hr / 24)
  if (days < 30) return `${days}d ago`
  return d.toLocaleDateString()
}

const getProviderTypeMeta = (type: string) => {
  const normalized = type.toUpperCase()
  return PROVIDER_TYPES.find((t) => t.key === normalized) ?? PROVIDER_TYPES[0]
}

export default function PageOverview({
  onCreateProvider,
  onDeleteProvider,
  onViewProvider,
  providers = [],
  isLoading,
  confirm,
  onConfirmClose,
}: PageOverviewProps) {
  const [query, setQuery] = useState('')
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = providers.length
    const active = providers.filter((p) => p.status === 'active').length
    const counts = PROVIDER_TYPES.reduce<Record<ProviderType, number>>(
      (acc, t) => {
        acc[t.key] = 0
        return acc
      },
      {} as Record<ProviderType, number>,
    )
    for (const prov of providers) {
      const normalized = prov.type.toUpperCase() as ProviderType
      if (counts[normalized] !== undefined) {
        counts[normalized] += 1
      }
    }
    return { total, active, ...counts }
  }, [providers])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = providers.filter((prov) => {
      const normalizedType = prov.type.toUpperCase()
      if (typeFilter !== 'all' && normalizedType !== typeFilter) return false
      if (statusFilter !== 'all' && prov.status !== statusFilter) return false
      if (!q) return true
      const hay = [prov.name, prov.type, prov.connection].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })
    list = [...list]
    if (sort === 'name') list.sort((a, b) => a.name.localeCompare(b.name))
    return list
  }, [providers, query, typeFilter, statusFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-4 sm:p-6 md:p-8 lg:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>User Federation</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Connect external user directories like LDAP and Kerberos to sync and authenticate users.
          </p>
        </div>
        <button
          onClick={() => onCreateProvider('LDAP')}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          Add Provider
        </button>
      </div>

      {/* Stats by type */}
      <div className='grid grid-cols-2 lg:grid-cols-4 gap-3'>
        <button
          onClick={() => setTypeFilter('all')}
          className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
            typeFilter === 'all'
              ? 'border-primary ring-1 ring-primary/30'
              : 'border-border hover:border-primary/30'
          }`}
        >
          <div className='h-9 w-9 rounded-md bg-primary/10 flex items-center justify-center'>
            <Database className='h-4 w-4 text-primary' />
          </div>
          <div>
            <div className='text-xl font-semibold tabular-nums leading-none'>
              {isLoading ? <Skeleton className='h-5 w-10' /> : stats.total}
            </div>
            <p className='text-xs text-muted-foreground mt-1'>All providers</p>
          </div>
        </button>
        {PROVIDER_TYPES.map((t) => {
          const tone = PROVIDER_TONE[t.key]
          const active = typeFilter === t.key
          return (
            <button
              key={t.key}
              onClick={() => setTypeFilter(t.key)}
              className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
                active
                  ? `${tone.border} ring-1 ${tone.border.replace('border-', 'ring-')}`
                  : 'border-border hover:border-primary/30'
              }`}
            >
              <div className={`h-9 w-9 rounded-md flex items-center justify-center ${tone.bg}`}>
                <t.icon className={`h-4 w-4 ${tone.fg}`} />
              </div>
              <div>
                <div className='text-xl font-semibold tabular-nums leading-none'>
                  {isLoading ? <Skeleton className='h-5 w-10' /> : stats[t.key]}
                </div>
                <p className='text-xs text-muted-foreground mt-1 truncate'>{t.short}</p>
              </div>
            </button>
          )
        })}
        <button
          onClick={() => {
            setTypeFilter('all')
            setStatusFilter('active')
          }}
          className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
            statusFilter === 'active' && typeFilter === 'all'
              ? 'border-emerald-500/40 ring-1 ring-emerald-500/40'
              : 'border-border hover:border-primary/30'
          }`}
        >
          <div className='h-9 w-9 rounded-md bg-emerald-500/10 flex items-center justify-center'>
            <CheckCircle2 className='h-4 w-4 text-emerald-500' />
          </div>
          <div>
            <div className='text-xl font-semibold tabular-nums leading-none'>
              {isLoading ? <Skeleton className='h-5 w-10' /> : stats.active}
            </div>
            <p className='text-xs text-muted-foreground mt-1'>Active</p>
          </div>
        </button>
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
          <div className='relative flex-1 sm:flex-none'>
            <Search className='pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search providers…'
              className='w-full sm:w-64 rounded-md border border-border bg-background pl-9 pr-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </div>
          <div className='relative shrink-0'>
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
            <div
              key={`prov-skel-${i}`}
              className='rounded-md border border-border bg-card/40 p-5 space-y-3'
            >
              <div className='flex items-center gap-3'>
                <Skeleton className='h-10 w-10 rounded-md' />
                <div className='space-y-2 flex-1'>
                  <Skeleton className='h-3 w-32' />
                  <Skeleton className='h-2 w-20' />
                </div>
              </div>
              <Skeleton className='h-3 w-full' />
            </div>
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className='rounded-md border border-dashed border-border bg-muted/20 p-12 flex flex-col items-center justify-center text-center gap-3'>
          <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
            <Database className='h-6 w-6 text-muted-foreground' />
          </div>
          <div>
            <p className='text-sm font-medium'>
              {providers.length === 0
                ? 'Connect your first user directory'
                : 'No providers match your filters'}
            </p>
            <p className='text-xs text-muted-foreground mt-1'>
              {providers.length === 0
                ? 'Sync users from LDAP or Kerberos directories to enable external authentication.'
                : 'Adjust the search, type or status filter.'}
            </p>
          </div>
          {providers.length === 0 && (
            <div className='flex items-center gap-3 mt-2'>
              <button
                onClick={() => onCreateProvider('LDAP')}
                className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
              >
                <Server className='h-4 w-4' />
                Add LDAP
              </button>
              <button
                onClick={() => onCreateProvider('Kerberos')}
                className='inline-flex items-center gap-2 rounded-md border border-border bg-background px-3.5 py-2 text-sm font-medium hover:bg-muted transition-colors'
              >
                <Key className='h-4 w-4' />
                Add Kerberos
              </button>
            </div>
          )}
        </div>
      ) : (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {filtered.map((prov) => {
            const meta = getProviderTypeMeta(prov.type)
            const tone = PROVIDER_TONE[prov.type.toUpperCase() as ProviderType] ?? PROVIDER_TONE.LDAP
            const statusTone = STATUS_TONE[prov.status === 'active' ? 'active' : 'inactive']
            return (
              <div
                key={prov.id}
                onClick={() => onViewProvider(prov.id, prov.type)}
                className='group flex flex-col gap-4 rounded-md border border-border bg-card/40 p-5 text-left transition hover:border-primary/30 hover:bg-muted/40 hover:shadow-sm cursor-pointer'
              >
                <div className='flex items-start gap-3'>
                  <div
                    className={`h-10 w-10 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}
                  >
                    <meta.icon className={`h-5 w-5 ${tone.fg}`} />
                  </div>
                  <div className='flex-1 min-w-0'>
                    <span className='text-sm font-medium truncate block group-hover:text-primary transition-colors'>
                      {prov.name}
                    </span>
                    <p className='text-xs text-muted-foreground truncate'>{prov.connection}</p>
                  </div>
                  <div className='flex items-center gap-1 shrink-0'>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        onViewProvider(prov.id, prov.type)
                      }}
                      className='p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors'
                      title='Edit'
                    >
                      <Pencil className='h-3.5 w-3.5' />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        onDeleteProvider(prov.id)
                      }}
                      className='p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors'
                      title='Delete'
                    >
                      <Trash2 className='h-3.5 w-3.5' />
                    </button>
                  </div>
                </div>

                <div className='flex items-center gap-2 mt-auto pt-3 border-t border-border/60 text-xs'>
                  <span
                    className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}
                  >
                    {meta.short}
                  </span>
                  <span
                    className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium ${statusTone.bg} ${statusTone.fg}`}
                  >
                    {prov.status === 'active' ? (
                      <CheckCircle2 className='h-2.5 w-2.5' />
                    ) : (
                      <XCircle className='h-2.5 w-2.5' />
                    )}
                    {prov.status}
                  </span>
                  <span className='inline-flex items-center gap-1 text-muted-foreground'>
                    <Users className='h-3 w-3' />
                    {prov.users.toLocaleString()}
                  </span>
                  <span className='ml-auto text-muted-foreground whitespace-nowrap inline-flex items-center gap-1'>
                    <Clock className='h-3 w-3' />
                    {formatRelative(prov.lastSync)}
                  </span>
                </div>
              </div>
            )
          })}
        </div>
      )}

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
