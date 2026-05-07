import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { isServiceAccount } from '@/utils'
import {
  AlertTriangle,
  CheckCircle2,
  ChevronDown,
  Plus,
  Search,
  ShieldOff,
  UserCog,
  UserRoundX,
  Users,
} from 'lucide-react'
import { useMemo, useState } from 'react'

import User = Schemas.User

interface Props {
  identities: User[]
  isLoading: boolean
  onSelect: (userId: string) => void
  onCreate: () => void
}

type StatusFilter = 'all' | 'active' | 'disabled' | 'unverified' | 'pending_actions'

const filters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'active', label: 'Active' },
  { key: 'unverified', label: 'Unverified' },
  { key: 'pending_actions', label: 'Pending actions' },
  { key: 'disabled', label: 'Disabled' },
]

type SortKey = 'recent' | 'name' | 'email'
const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
  email: 'Email (A→Z)',
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

const initials = (u: User) => {
  const f = (u.firstname ?? '').trim()
  const l = (u.lastname ?? '').trim()
  if (f || l) return `${f[0] ?? ''}${l[0] ?? ''}`.toUpperCase() || u.username[0]?.toUpperCase()
  return u.username.slice(0, 2).toUpperCase()
}

const displayName = (u: User) => {
  if (isServiceAccount(u)) return 'Service account'
  const full = [u.firstname, u.lastname].filter(Boolean).join(' ')
  return full || u.username
}

export default function PageIdentities({ identities, isLoading, onSelect, onCreate }: Props) {
  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = identities.length
    const active = identities.filter((u) => u.enabled).length
    const verified = identities.filter((u) => u.email_verified).length
    const pending = identities.filter((u) => (u.required_actions?.length ?? 0) > 0).length
    return { total, active, verified, pending }
  }, [identities])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = identities.filter((u) => {
      if (statusFilter === 'active' && !u.enabled) return false
      if (statusFilter === 'disabled' && u.enabled) return false
      if (statusFilter === 'unverified' && u.email_verified) return false
      if (statusFilter === 'pending_actions' && (u.required_actions?.length ?? 0) === 0) return false
      if (!q) return true
      const hay = [u.username, u.email, u.firstname, u.lastname].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })

    list = [...list]
    if (sort === 'recent') list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    if (sort === 'name') list.sort((a, b) => displayName(a).localeCompare(displayName(b)))
    if (sort === 'email') list.sort((a, b) => (a.email ?? '').localeCompare(b.email ?? ''))
    return list
  }, [identities, query, statusFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-8 md:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Identities</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Customer accounts in this realm. Search, filter and manage sign-in credentials.
          </p>
        </div>
        <button
          onClick={onCreate}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          Create identity
        </button>
      </div>

      {/* Lightweight stat row */}
      <div className='grid grid-cols-2 lg:grid-cols-4 gap-4'>
        <SmallStat
          icon={Users}
          tone='emerald'
          label='Total identities'
          value={stats.total}
          isLoading={isLoading}
        />
        <SmallStat
          icon={CheckCircle2}
          tone='blue'
          label='Email verified'
          value={stats.verified}
          isLoading={isLoading}
        />
        <SmallStat
          icon={UserCog}
          tone='amber'
          label='Pending actions'
          value={stats.pending}
          isLoading={isLoading}
        />
        <SmallStat
          icon={UserRoundX}
          tone='muted'
          label='Disabled'
          value={stats.total - stats.active}
          isLoading={isLoading}
        />
      </div>

      {/* Toolbar */}
      <div className='flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
        <div className='flex items-center gap-2 flex-wrap'>
          {filters.map((f) => (
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
              placeholder='Search by name, email, username…'
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

      {/* List */}
      <div className='rounded-md border border-border bg-card/40 overflow-hidden'>
        {/* Header row */}
        <div className='hidden md:grid grid-cols-[minmax(0,2.2fr)_minmax(0,1.6fr)_140px_140px_60px] gap-4 px-5 py-2.5 border-b border-border bg-muted/30 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
          <span>Identity</span>
          <span>Email</span>
          <span>Status</span>
          <span>Created</span>
          <span className='text-right'>Roles</span>
        </div>

        {isLoading ? (
          <div className='divide-y divide-border'>
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={`row-skel-${i}`} className='grid grid-cols-[minmax(0,2.2fr)_minmax(0,1.6fr)_140px_140px_60px] gap-4 items-center px-5 py-3'>
                <div className='flex items-center gap-3'>
                  <Skeleton className='h-9 w-9 rounded-md' />
                  <div className='space-y-1.5 flex-1'>
                    <Skeleton className='h-3 w-32' />
                    <Skeleton className='h-2 w-20' />
                  </div>
                </div>
                <Skeleton className='h-3 w-40' />
                <Skeleton className='h-5 w-20 rounded-md' />
                <Skeleton className='h-3 w-16' />
                <Skeleton className='h-3 w-6 ml-auto' />
              </div>
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className='flex flex-col items-center justify-center py-16 text-center gap-3'>
            <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
              <Users className='h-6 w-6 text-muted-foreground' />
            </div>
            <div>
              <p className='text-sm font-medium'>No identities match your filters</p>
              <p className='text-xs text-muted-foreground mt-1'>
                Adjust the search or invite the first identity.
              </p>
            </div>
          </div>
        ) : (
          <div className='divide-y divide-border'>
            {filtered.map((u) => {
              const sa = isServiceAccount(u)
              const pendingCount = u.required_actions?.length ?? 0
              const rolesCount = u.roles?.length ?? 0
              return (
                <button
                  key={u.id}
                  onClick={() => onSelect(u.id)}
                  className='w-full grid md:grid-cols-[minmax(0,2.2fr)_minmax(0,1.6fr)_140px_140px_60px] gap-4 items-center px-5 py-3 text-left hover:bg-muted/40 transition-colors'
                >
                  {/* Identity */}
                  <div className='flex items-center gap-3 min-w-0'>
                    <div className={`h-9 w-9 rounded-md flex items-center justify-center text-xs font-semibold ${sa ? 'bg-violet-500/10 text-violet-500' : 'bg-primary/10 text-primary'}`}>
                      {sa ? 'S' : initials(u)}
                    </div>
                    <div className='min-w-0 flex-1'>
                      <div className='flex items-center gap-2'>
                        <span className='text-sm font-medium truncate'>{displayName(u)}</span>
                        {sa && (
                          <span className='text-[10px] font-medium px-1.5 py-0.5 rounded bg-violet-500/10 text-violet-500 border border-violet-500/30 uppercase tracking-wide'>
                            Service
                          </span>
                        )}
                      </div>
                      <p className='text-xs text-muted-foreground truncate'>@{u.username}</p>
                    </div>
                  </div>

                  {/* Email */}
                  <div className='min-w-0 hidden md:flex items-center gap-2'>
                    <span className='text-sm truncate'>{u.email ?? '—'}</span>
                    {u.email && !u.email_verified && (
                      <span title='Email not verified' className='inline-flex items-center'>
                        <AlertTriangle className='h-3.5 w-3.5 text-amber-500' />
                      </span>
                    )}
                    {u.email && u.email_verified && (
                      <span title='Verified' className='inline-flex items-center'>
                        <CheckCircle2 className='h-3.5 w-3.5 text-emerald-500' />
                      </span>
                    )}
                  </div>

                  {/* Status */}
                  <div className='hidden md:block'>
                    {!u.enabled ? (
                      <StatusPill tone='muted' icon={ShieldOff} label='Disabled' />
                    ) : pendingCount > 0 ? (
                      <StatusPill tone='amber' icon={UserCog} label={`${pendingCount} action${pendingCount > 1 ? 's' : ''}`} />
                    ) : (
                      <StatusPill tone='emerald' icon={CheckCircle2} label='Active' />
                    )}
                  </div>

                  {/* Created */}
                  <div className='hidden md:block text-xs text-muted-foreground'>
                    {formatRelative(u.created_at)}
                  </div>

                  {/* Roles */}
                  <div className='hidden md:block text-xs text-muted-foreground tabular-nums text-right'>
                    {rolesCount}
                  </div>
                </button>
              )
            })}
          </div>
        )}

        {!isLoading && filtered.length > 0 && (
          <div className='flex items-center justify-between border-t border-border bg-muted/20 px-5 py-2 text-xs text-muted-foreground'>
            <span>
              Showing {filtered.length} of {identities.length}
            </span>
            <span>{stats.verified} verified · {stats.pending} pending action(s)</span>
          </div>
        )}
      </div>
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

interface StatusPillProps {
  tone: 'emerald' | 'amber' | 'muted'
  icon: React.ComponentType<{ className?: string }>
  label: string
}

function StatusPill({ tone, icon: Icon, label }: StatusPillProps) {
  const tones: Record<StatusPillProps['tone'], string> = {
    emerald: 'bg-emerald-500/10 text-emerald-600 border border-emerald-500/30',
    amber: 'bg-amber-500/10 text-amber-600 border border-amber-500/30',
    muted: 'bg-muted text-muted-foreground border border-border',
  }
  return (
    <span className={`inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium ${tones[tone]}`}>
      <Icon className='h-3 w-3' />
      {label}
    </span>
  )
}
