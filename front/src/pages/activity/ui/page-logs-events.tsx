import { Schemas } from '@/api/api.client'
import { Skeleton } from '@/components/ui/skeleton'
import { formatSnakeCaseToTitleCase } from '@/utils'
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  ChevronDown,
  Clock,
  Globe,
  Monitor,
  Search,
  ShieldAlert,
  ShieldCheck,
  Smartphone,
  User,
  XCircle,
} from 'lucide-react'
import { useMemo, useState } from 'react'

import SecurityEvent = Schemas.SecurityEvent

interface Props {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  isMocked: boolean
}

type StatusFilter = 'all' | 'success' | 'failure'
type SortKey = 'recent' | 'oldest'

const filters: { key: StatusFilter; label: string }[] = [
  { key: 'all', label: 'All events' },
  { key: 'success', label: 'Success' },
  { key: 'failure', label: 'Failures' },
]

const sortLabels: Record<SortKey, string> = {
  recent: 'Most recent',
  oldest: 'Oldest first',
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

const formatAbsolute = (iso: string) => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

const getDeviceIcon = (userAgent?: string | null) => {
  if (!userAgent) return Monitor
  const lower = userAgent.toLowerCase()
  if (lower.includes('iphone') || lower.includes('android') || lower.includes('mobile')) {
    return Smartphone
  }
  return Monitor
}

const getActorLabel = (event: SecurityEvent) => {
  return event.actor_id ?? event.target_id ?? 'Unknown'
}

export default function PageLogsEvents({ events, isLoading, isError, isMocked }: Props) {
  const [query, setQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [sort, setSort] = useState<SortKey>('recent')
  const [expanded, setExpanded] = useState<string | null>(null)

  const stats = useMemo(() => {
    const total = events.length
    const failures = events.filter((e) => e.status === 'failure').length
    const successes = total - failures
    const actors = new Set(events.map((e) => e.actor_id).filter(Boolean))
    return {
      total,
      failures,
      successes,
      uniqueActors: actors.size,
    }
  }, [events])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    let list = events.filter((event) => {
      if (statusFilter !== 'all' && event.status !== statusFilter) return false
      if (!q) return true
      const haystack = [
        event.event_type,
        event.actor_id,
        event.target_id,
        event.target_type,
        event.resource,
        event.ip_address,
        event.user_agent,
        event.status,
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      return haystack.includes(q)
    })

    list = [...list]
    if (sort === 'recent') {
      list.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
    } else {
      list.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())
    }
    return list
  }, [events, query, statusFilter, sort])

  return (
    <div className='flex flex-col gap-6 p-4 sm:p-6 md:p-8 lg:p-12'>
      {/* Header */}
      <div className='flex flex-col gap-2 md:flex-row md:items-start md:justify-between'>
        <div>
          <h1 className='text-2xl font-medium tracking-tight'>Logs &amp; events</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Searchable feed of authentication events, errors and admin actions in this realm.
          </p>
        </div>
        {isMocked && !isLoading && (
          <span className='inline-flex items-center gap-1.5 self-start rounded-md border border-amber-500/30 bg-amber-500/10 px-2.5 py-1 text-xs font-medium text-amber-600'>
            <AlertTriangle className='h-3 w-3' />
            Showing sample data
          </span>
        )}
      </div>

      {/* Stat row */}
      <div className='grid grid-cols-2 lg:grid-cols-4 gap-4'>
        <SmallStat
          icon={Activity}
          tone='blue'
          label='Total events'
          value={stats.total}
          isLoading={isLoading}
        />
        <SmallStat
          icon={ShieldCheck}
          tone='emerald'
          label='Successes'
          value={stats.successes}
          isLoading={isLoading}
        />
        <SmallStat
          icon={ShieldAlert}
          tone='red'
          label='Failures'
          value={stats.failures}
          isLoading={isLoading}
        />
        <SmallStat
          icon={User}
          tone='muted'
          label='Unique actors'
          value={stats.uniqueActors}
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
              className={`px-3 sm:px-3.5 py-1.5 rounded-md text-xs font-medium transition-colors border ${
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
              placeholder='Search by event, actor, IP…'
              className='w-full sm:w-72 rounded-md border border-border bg-background pl-9 pr-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
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

      {/* Error banner */}
      {isError && (
        <div className='flex items-start gap-3 rounded-md border border-amber-500/30 bg-amber-500/5 px-4 py-3 text-sm'>
          <AlertTriangle className='h-4 w-4 text-amber-500 mt-0.5 shrink-0' />
          <div>
            <p className='font-medium text-amber-600'>Live feed unavailable</p>
            <p className='text-xs text-muted-foreground mt-0.5'>
              We couldn&apos;t fetch the latest events. Displaying the most recent cached data.
            </p>
          </div>
        </div>
      )}

      {/* List */}
      <div className='rounded-md border border-border bg-card/40 overflow-hidden'>
        <div className='hidden md:grid grid-cols-[minmax(0,2fr)_minmax(0,1.5fr)_120px_140px_140px_24px] gap-4 px-5 py-2.5 border-b border-border bg-muted/30 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
          <span>Event</span>
          <span>Actor</span>
          <span>Status</span>
          <span>Source</span>
          <span>When</span>
          <span />
        </div>

        {isLoading ? (
          <div className='divide-y divide-border'>
            {Array.from({ length: 6 }).map((_, i) => (
              <div
                key={`evt-skel-${i}`}
                className='flex md:grid md:grid-cols-[minmax(0,2fr)_minmax(0,1.5fr)_120px_140px_140px_24px] gap-4 items-center px-4 sm:px-5 py-3'
              >
                <div className='flex items-center gap-3 flex-1 md:flex-none'>
                  <Skeleton className='h-8 w-8 rounded-md' />
                  <div className='space-y-1.5 flex-1'>
                    <Skeleton className='h-3 w-40' />
                    <Skeleton className='h-2 w-24' />
                  </div>
                </div>
                <Skeleton className='hidden md:block h-3 w-32' />
                <Skeleton className='hidden md:block h-5 w-16 rounded-md' />
                <Skeleton className='hidden md:block h-3 w-24' />
                <Skeleton className='hidden md:block h-3 w-20' />
                <Skeleton className='hidden md:block h-3 w-3' />
              </div>
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className='flex flex-col items-center justify-center py-16 text-center gap-3'>
            <div className='h-12 w-12 rounded-md bg-muted flex items-center justify-center'>
              <Activity className='h-6 w-6 text-muted-foreground' />
            </div>
            <div>
              <p className='text-sm font-medium'>No events match your filters</p>
              <p className='text-xs text-muted-foreground mt-1'>
                Try a different status or clear your search.
              </p>
            </div>
          </div>
        ) : (
          <div className='divide-y divide-border'>
            {filtered.map((event) => {
              const isOpen = expanded === event.id
              const DeviceIcon = getDeviceIcon(event.user_agent)
              const actorLabel = getActorLabel(event)
              const hasDetails =
                event.details && typeof event.details === 'object' && event.details !== null

              return (
                <div key={event.id}>
                  <button
                    onClick={() => setExpanded(isOpen ? null : event.id)}
                    className='w-full flex flex-col md:grid md:grid-cols-[minmax(0,2fr)_minmax(0,1.5fr)_120px_140px_140px_24px] gap-3 md:gap-4 md:items-center px-4 sm:px-5 py-3 text-left hover:bg-muted/40 transition-colors'
                  >
                    {/* Event title row */}
                    <div className='flex items-center gap-3 min-w-0'>
                      <div
                        className={`h-8 w-8 rounded-md flex items-center justify-center shrink-0 ${
                          event.status === 'failure'
                            ? 'bg-red-500/10 text-red-500'
                            : 'bg-emerald-500/10 text-emerald-500'
                        }`}
                      >
                        {event.status === 'failure' ? (
                          <XCircle className='h-4 w-4' />
                        ) : (
                          <CheckCircle2 className='h-4 w-4' />
                        )}
                      </div>
                      <div className='min-w-0 flex-1'>
                        <span className='text-sm font-medium truncate block'>
                          {formatSnakeCaseToTitleCase(event.event_type)}
                        </span>
                        {event.resource && (
                          <p className='text-xs text-muted-foreground truncate'>
                            on {event.resource}
                          </p>
                        )}
                      </div>
                      {/* Mobile-only status + chevron on the right of title */}
                      <div className='md:hidden flex items-center gap-2 shrink-0'>
                        {event.status === 'failure' ? (
                          <StatusPill tone='red' icon={XCircle} label='Failure' />
                        ) : (
                          <StatusPill tone='emerald' icon={CheckCircle2} label='Success' />
                        )}
                        <ChevronDown
                          className={`h-3.5 w-3.5 text-muted-foreground transition-transform ${
                            isOpen ? 'rotate-180' : ''
                          }`}
                        />
                      </div>
                    </div>

                    {/* Mobile: stacked metadata under title */}
                    <div className='md:hidden flex flex-col gap-1 text-xs text-muted-foreground pl-11'>
                      <span className='inline-flex items-center gap-1.5 truncate'>
                        <User className='h-3 w-3 shrink-0' />
                        <span className='truncate'>{actorLabel}</span>
                      </span>
                      {event.ip_address && (
                        <span className='inline-flex items-center gap-1.5 truncate'>
                          <Globe className='h-3 w-3 shrink-0' />
                          <span className='truncate tabular-nums'>{event.ip_address}</span>
                        </span>
                      )}
                      <span
                        className='inline-flex items-center gap-1.5'
                        title={formatAbsolute(event.timestamp)}
                      >
                        <Clock className='h-3 w-3' />
                        {formatRelative(event.timestamp)}
                      </span>
                    </div>

                    {/* Actor (desktop) */}
                    <div className='hidden md:flex items-center gap-2 min-w-0'>
                      <User className='h-3.5 w-3.5 text-muted-foreground shrink-0' />
                      <span className='text-sm truncate'>{actorLabel}</span>
                    </div>

                    {/* Status (desktop) */}
                    <div className='hidden md:block'>
                      {event.status === 'failure' ? (
                        <StatusPill tone='red' icon={XCircle} label='Failure' />
                      ) : (
                        <StatusPill tone='emerald' icon={CheckCircle2} label='Success' />
                      )}
                    </div>

                    {/* Source (desktop) */}
                    <div className='hidden md:flex flex-col gap-0.5 text-xs text-muted-foreground min-w-0'>
                      {event.ip_address && (
                        <span className='inline-flex items-center gap-1 truncate'>
                          <Globe className='h-3 w-3 shrink-0' />
                          <span className='truncate tabular-nums'>{event.ip_address}</span>
                        </span>
                      )}
                      {event.user_agent && (
                        <span className='inline-flex items-center gap-1 truncate'>
                          <DeviceIcon className='h-3 w-3 shrink-0' />
                          <span className='truncate'>{event.user_agent}</span>
                        </span>
                      )}
                    </div>

                    {/* When (desktop) */}
                    <div
                      className='hidden md:flex items-center gap-1 text-xs text-muted-foreground'
                      title={formatAbsolute(event.timestamp)}
                    >
                      <Clock className='h-3 w-3' />
                      {formatRelative(event.timestamp)}
                    </div>

                    {/* Chevron (desktop) */}
                    <ChevronDown
                      className={`hidden md:block h-3.5 w-3.5 text-muted-foreground transition-transform ${
                        isOpen ? 'rotate-180' : ''
                      }`}
                    />
                  </button>

                  {isOpen && (
                    <div className='border-t border-border bg-muted/20 px-4 sm:px-5 py-4 text-xs space-y-3'>
                      <div className='grid grid-cols-2 md:grid-cols-4 gap-x-6 gap-y-2'>
                        <DetailField label='Event ID' value={event.id} mono />
                        <DetailField label='Timestamp' value={formatAbsolute(event.timestamp)} />
                        {event.target_id && (
                          <DetailField label='Target' value={event.target_id} mono />
                        )}
                        {event.target_type && (
                          <DetailField label='Target type' value={event.target_type} />
                        )}
                      </div>
                      {hasDetails && (
                        <div className='border-t border-border pt-3'>
                          <p className='text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mb-2'>
                            Details
                          </p>
                          <div className='flex flex-col gap-1.5'>
                            {Object.entries(event.details as Record<string, unknown>).map(
                              ([key, value]) => (
                                <div key={key} className='flex items-start gap-2'>
                                  <span className='text-muted-foreground min-w-[120px]'>
                                    {formatSnakeCaseToTitleCase(key)}
                                  </span>
                                  <code className='text-foreground bg-muted px-1.5 py-0.5 rounded text-[11px] break-all'>
                                    {String(value)}
                                  </code>
                                </div>
                              ),
                            )}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}

        {!isLoading && filtered.length > 0 && (
          <div className='flex items-center justify-between gap-2 border-t border-border bg-muted/20 px-4 sm:px-5 py-2 text-xs text-muted-foreground'>
            <span>
              Showing {filtered.length} of {events.length}
            </span>
            <span>
              {stats.successes} success · {stats.failures} failure(s)
            </span>
          </div>
        )}
      </div>
    </div>
  )
}

interface SmallStatProps {
  icon: React.ComponentType<{ className?: string }>
  tone: 'emerald' | 'blue' | 'red' | 'muted'
  label: string
  value: number
  isLoading: boolean
}

function SmallStat({ icon: Icon, tone, label, value, isLoading }: SmallStatProps) {
  const tones: Record<SmallStatProps['tone'], { bg: string; fg: string }> = {
    emerald: { bg: 'bg-emerald-500/10', fg: 'text-emerald-500' },
    blue: { bg: 'bg-blue-500/10', fg: 'text-blue-500' },
    red: { bg: 'bg-red-500/10', fg: 'text-red-500' },
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
  tone: 'emerald' | 'red'
  icon: React.ComponentType<{ className?: string }>
  label: string
}

function StatusPill({ tone, icon: Icon, label }: StatusPillProps) {
  const tones: Record<StatusPillProps['tone'], string> = {
    emerald: 'bg-emerald-500/10 text-emerald-600 border border-emerald-500/30',
    red: 'bg-red-500/10 text-red-600 border border-red-500/30',
  }
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium ${tones[tone]}`}
    >
      <Icon className='h-3 w-3' />
      {label}
    </span>
  )
}

interface DetailFieldProps {
  label: string
  value: string
  mono?: boolean
}

function DetailField({ label, value, mono }: DetailFieldProps) {
  return (
    <div className='flex flex-col gap-0.5 min-w-0'>
      <span className='text-[11px] font-semibold uppercase tracking-wider text-muted-foreground'>
        {label}
      </span>
      <span className={`text-xs truncate ${mono ? 'font-mono' : ''}`} title={value}>
        {value}
      </span>
    </div>
  )
}
