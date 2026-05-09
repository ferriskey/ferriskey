import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import {
  ChevronDown,
  ChevronRight,
  KeySquare,
  Lock,
  Plus,
  Search,
  ShieldCheck,
  ShieldUser,
} from 'lucide-react'
import { useMemo, useState } from 'react'

import Role = Schemas.Role

interface Props {
  roles: Role[]
  isLoading: boolean
  onSelect: (roleId: string) => void
  onCreate: () => void
}

type ScopeFilter = 'all' | 'realm' | 'client'

const filters: { key: ScopeFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'realm', label: 'Realm-level' },
  { key: 'client', label: 'Client-level' },
]

type SortKey = 'recent' | 'name' | 'permissions'
const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
  permissions: 'Most permissions',
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

export default function PageRoles({ roles, isLoading, onSelect, onCreate }: Props) {
  const [query, setQuery] = useState('')
  const [scopeFilter, setScopeFilter] = useState<ScopeFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = roles.length
    const realmRoles = roles.filter((r) => !r.client_id).length
    const clientRoles = roles.filter((r) => !!r.client_id).length
    const allPermissions = new Set(roles.flatMap((r) => r.permissions ?? []))
    return { total, realmRoles, clientRoles, distinctPermissions: allPermissions.size }
  }, [roles])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = roles.filter((r) => {
      if (scopeFilter === 'realm' && r.client_id) return false
      if (scopeFilter === 'client' && !r.client_id) return false
      if (!q) return true
      const hay = [r.name, r.description, ...(r.permissions ?? [])].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })
    list = [...list]
    if (sort === 'recent') list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    if (sort === 'name') list.sort((a, b) => a.name.localeCompare(b.name))
    if (sort === 'permissions')
      list.sort((a, b) => (b.permissions?.length ?? 0) - (a.permissions?.length ?? 0))
    return list
  }, [roles, query, scopeFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-8 md:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Roles</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Group permissions into reusable roles. Assign them to identities to grant access.
          </p>
        </div>
        <button
          onClick={onCreate}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          Create role
        </button>
      </div>

      {/* Stats */}
      <div className='grid grid-cols-2 lg:grid-cols-4 gap-4'>
        <SmallStat icon={ShieldUser} tone='emerald' label='Total roles' value={stats.total} isLoading={isLoading} />
        <SmallStat icon={ShieldCheck} tone='blue' label='Realm-level' value={stats.realmRoles} isLoading={isLoading} />
        <SmallStat icon={Lock} tone='violet' label='Client-level' value={stats.clientRoles} isLoading={isLoading} />
        <SmallStat icon={KeySquare} tone='amber' label='Distinct permissions' value={stats.distinctPermissions} isLoading={isLoading} />
      </div>

      {/* Toolbar */}
      <div className='flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between'>
        <div className='flex items-center gap-2 flex-wrap'>
          {filters.map((f) => (
            <button
              key={f.key}
              onClick={() => setScopeFilter(f.key)}
              className={`px-3.5 py-1.5 rounded-md text-xs font-medium transition-colors border ${
                scopeFilter === f.key
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
              placeholder='Search by name, description, permission…'
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
        <div className='hidden md:grid grid-cols-[minmax(0,2fr)_140px_minmax(0,1.5fr)_140px_24px] gap-4 px-5 py-2.5 border-b border-border bg-muted/30 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
          <span>Role</span>
          <span>Scope</span>
          <span>Permissions</span>
          <span>Created</span>
          <span />
        </div>

        {isLoading ? (
          <div className='divide-y divide-border'>
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={`role-skel-${i}`} className='grid grid-cols-[minmax(0,2fr)_140px_minmax(0,1.5fr)_140px_24px] gap-4 items-center px-5 py-3.5'>
                <div className='flex items-center gap-3'>
                  <Skeleton className='h-9 w-9 rounded-md' />
                  <div className='space-y-1.5 flex-1'>
                    <Skeleton className='h-3 w-32' />
                    <Skeleton className='h-2 w-40' />
                  </div>
                </div>
                <Skeleton className='h-5 w-20 rounded-md' />
                <Skeleton className='h-3 w-32' />
                <Skeleton className='h-3 w-16' />
                <Skeleton className='h-3 w-3' />
              </div>
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className='flex flex-col items-center justify-center py-16 text-center gap-3'>
            <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
              <ShieldUser className='h-6 w-6 text-muted-foreground' />
            </div>
            <div>
              <p className='text-sm font-medium'>No roles match your filters</p>
              <p className='text-xs text-muted-foreground mt-1'>
                {roles.length === 0 ? 'Create your first role to define permissions.' : 'Adjust the search or scope filter.'}
              </p>
            </div>
          </div>
        ) : (
          <div className='divide-y divide-border'>
            {filtered.map((r) => {
              const isClientScope = !!r.client_id
              const permCount = r.permissions?.length ?? 0
              const previewPerms = (r.permissions ?? []).slice(0, 2)
              return (
                <button
                  key={r.id}
                  onClick={() => onSelect(r.id)}
                  className='group w-full grid md:grid-cols-[minmax(0,2fr)_140px_minmax(0,1.5fr)_140px_24px] gap-4 items-center px-5 py-3.5 text-left hover:bg-muted/40 transition-colors'
                >
                  {/* Role */}
                  <div className='flex items-center gap-3 min-w-0'>
                    <div className={`h-9 w-9 rounded-md flex items-center justify-center ${isClientScope ? 'bg-violet-500/10 text-violet-500' : 'bg-primary/10 text-primary'}`}>
                      <ShieldUser className='h-4 w-4' />
                    </div>
                    <div className='min-w-0 flex-1'>
                      <span className='text-sm font-medium truncate block group-hover:text-primary transition-colors'>
                        {r.name}
                      </span>
                      <p className='text-xs text-muted-foreground truncate'>
                        {r.description || 'No description'}
                      </p>
                    </div>
                  </div>

                  {/* Scope */}
                  <div className='hidden md:block'>
                    {isClientScope ? (
                      <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-violet-500/10 text-violet-500 border border-violet-500/30'>
                        <Lock className='h-3 w-3' />
                        Client
                      </span>
                    ) : (
                      <span className='inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/30'>
                        <ShieldCheck className='h-3 w-3' />
                        Realm
                      </span>
                    )}
                  </div>

                  {/* Permissions preview */}
                  <div className='hidden md:flex items-center gap-1 flex-wrap'>
                    {previewPerms.length === 0 ? (
                      <span className='text-xs text-muted-foreground/70'>No permissions</span>
                    ) : (
                      <>
                        {previewPerms.map((p) => (
                          <span
                            key={p}
                            className='inline-flex items-center rounded-md border border-border bg-background px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground'
                          >
                            {p}
                          </span>
                        ))}
                        {permCount > previewPerms.length && (
                          <span className='text-[11px] text-muted-foreground'>
                            +{permCount - previewPerms.length}
                          </span>
                        )}
                      </>
                    )}
                  </div>

                  {/* Created */}
                  <div className='hidden md:block text-xs text-muted-foreground'>
                    {formatRelative(r.created_at)}
                  </div>

                  <ChevronRight className='hidden md:block h-4 w-4 text-muted-foreground/50 group-hover:text-primary transition-colors' />
                </button>
              )
            })}
          </div>
        )}

        {!isLoading && filtered.length > 0 && (
          <div className='flex items-center justify-between border-t border-border bg-muted/20 px-5 py-2 text-xs text-muted-foreground'>
            <span>
              Showing {filtered.length} of {roles.length}
            </span>
            <span>
              {stats.realmRoles} realm · {stats.clientRoles} client-level
            </span>
          </div>
        )}
      </div>
    </div>
  )
}

interface SmallStatProps {
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue' | 'amber' | 'violet'
  label: string
  value: number
  isLoading: boolean
}

function SmallStat({ icon: Icon, tone, label, value, isLoading }: SmallStatProps) {
  const tones: Record<SmallStatProps['tone'], { bg: string; fg: string }> = {
    emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500' },
    blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500' },
    amber: { bg: 'bg-amber-500/10', fg: 'text-amber-500' },
    violet: { bg: 'bg-violet-500/10', fg: 'text-violet-500' },
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
