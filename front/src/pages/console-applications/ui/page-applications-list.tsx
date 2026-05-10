import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { ApplicationType } from '@/routes/sub-router/applications.router'
import { Boxes, ChevronDown, Plus, Search, ShieldOff } from 'lucide-react'
import { useMemo, useState } from 'react'
import {
  APPLICATION_TONE,
  APPLICATION_TYPES,
  getApplicationTypeMeta,
  inferApplicationType,
} from '../types'

import Client = Schemas.Client

interface Props {
  applications: Client[]
  isLoading: boolean
  onCreate: () => void
  onSelect: (clientId: string) => void
}

type StatusFilter = 'all' | 'enabled' | 'disabled'
const statusFilters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'enabled', label: 'Enabled' },
  { key: 'disabled', label: 'Disabled' },
]

type SortKey = 'recent' | 'name'
const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  name: 'Name (A→Z)',
}

const formatRelative = (iso: string) => {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
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

export default function PageApplicationsList({ applications, isLoading, onCreate, onSelect }: Props) {
  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [typeFilter, setTypeFilter] = useState<ApplicationType | 'all'>('all')
  const [sort, setSort] = useState<SortKey>('recent')

  const stats = useMemo(() => {
    const total = applications.length
    const counts = APPLICATION_TYPES.reduce<Record<ApplicationType, number>>(
      (acc, t) => {
        acc[t.key] = 0
        return acc
      },
      { native: 0, spa: 0, web: 0, m2m: 0 },
    )
    for (const c of applications) counts[inferApplicationType(c)] += 1
    return { total, ...counts }
  }, [applications])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = applications.filter((c) => {
      if (statusFilter === 'enabled' && !c.enabled) return false
      if (statusFilter === 'disabled' && c.enabled) return false
      if (typeFilter !== 'all' && inferApplicationType(c) !== typeFilter) return false
      if (!q) return true
      const hay = [c.name, c.client_id].filter(Boolean).join(' ').toLowerCase()
      return hay.includes(q)
    })
    list = [...list]
    if (sort === 'recent')
      list.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    if (sort === 'name') list.sort((a, b) => (a.name || a.client_id).localeCompare(b.name || b.client_id))
    return list
  }, [applications, query, statusFilter, typeFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-8 md:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Applications</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Anything that talks to FerrisKey: mobile apps, single-page apps, web servers, daemons.
          </p>
        </div>
        <button
          onClick={onCreate}
          className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors'
        >
          <Plus className='h-4 w-4' />
          Create application
        </button>
      </div>

      {/* Stats by type */}
      <div className='grid grid-cols-2 lg:grid-cols-5 gap-3'>
        <button
          onClick={() => setTypeFilter('all')}
          className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
            typeFilter === 'all' ? 'border-primary ring-1 ring-primary/30' : 'border-border hover:border-primary/30'
          }`}
        >
          <div className='h-9 w-9 rounded-md bg-primary/10 flex items-center justify-center'>
            <Boxes className='h-4 w-4 text-primary' />
          </div>
          <div>
            <div className='text-xl font-semibold tabular-nums leading-none'>
              {isLoading ? <Skeleton className='h-5 w-10' /> : stats.total}
            </div>
            <p className='text-xs text-muted-foreground mt-1'>All applications</p>
          </div>
        </button>
        {APPLICATION_TYPES.map((t) => {
          const tone = APPLICATION_TONE[t.tone]
          const active = typeFilter === t.key
          return (
            <button
              key={t.key}
              onClick={() => setTypeFilter(t.key)}
              className={`flex items-center gap-3 rounded-md border bg-card/40 px-4 py-3 text-left transition ${
                active ? `${tone.border} ring-1 ${tone.border.replace('border-', 'ring-')}` : 'border-border hover:border-primary/30'
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
              placeholder='Search by name or client ID…'
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
            <div key={`app-skel-${i}`} className='rounded-md border border-border bg-card/40 p-5 space-y-3'>
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
            <Boxes className='h-6 w-6 text-muted-foreground' />
          </div>
          <div>
            <p className='text-sm font-medium'>
              {applications.length === 0 ? 'Register your first application' : 'No applications match your filters'}
            </p>
            <p className='text-xs text-muted-foreground mt-1'>
              {applications.length === 0
                ? 'Pick a type, give it a name and we generate everything you need to integrate.'
                : 'Adjust the search, type or status filter.'}
            </p>
          </div>
          {applications.length === 0 && (
            <button
              onClick={onCreate}
              className='inline-flex items-center gap-2 rounded-md bg-primary px-3.5 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors mt-2'
            >
              <Plus className='h-4 w-4' />
              Create application
            </button>
          )}
        </div>
      ) : (
        <div className='grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4'>
          {filtered.map((c) => {
            const meta = getApplicationTypeMeta(inferApplicationType(c))
            const tone = APPLICATION_TONE[meta.tone]
            return (
              <button
                key={c.id}
                onClick={() => onSelect(c.id)}
                className='group flex flex-col gap-4 rounded-md border border-border bg-card/40 p-5 text-left transition hover:border-primary/30 hover:bg-muted/40 hover:shadow-sm'
              >
                <div className='flex items-start gap-3'>
                  <div className={`h-10 w-10 rounded-md flex items-center justify-center shrink-0 ${tone.bg}`}>
                    <meta.icon className={`h-5 w-5 ${tone.fg}`} />
                  </div>
                  <div className='flex-1 min-w-0'>
                    <div className='flex items-center gap-2'>
                      <span className='text-sm font-medium truncate group-hover:text-primary transition-colors'>
                        {c.name || c.client_id}
                      </span>
                      {!c.enabled && (
                        <span className='inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground border border-border uppercase tracking-wide'>
                          <ShieldOff className='h-2.5 w-2.5' />
                          Off
                        </span>
                      )}
                    </div>
                    <p className='text-xs text-muted-foreground font-mono truncate'>{c.client_id}</p>
                  </div>
                </div>

                <div className='flex items-center gap-2 mt-auto pt-3 border-t border-border/60 text-xs'>
                  <span className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${tone.bg} ${tone.fg}`}>
                    {meta.short}
                  </span>
                  <span className='text-muted-foreground truncate'>{meta.flow}</span>
                  <span className='ml-auto text-muted-foreground whitespace-nowrap'>{formatRelative(c.created_at)}</span>
                </div>
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}
