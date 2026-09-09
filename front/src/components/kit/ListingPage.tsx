import { useMemo, useState, type ReactNode } from 'react'
import { AlertTriangle, CheckCircle2, LayoutGrid, List, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { ChartTone } from './charts'
import { MetricsBand } from './MetricsBand'
import { DataView, type CardSpec, type Column, type ViewMode } from './DataView'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface ListingMetric {
  key: string
  label: string
  value: number | string
  hint?: string
  delta?: number
  series?: number[]
  tone?: ChartTone
}

export interface ListingAlert {
  tone: 'warn' | 'error' | 'ok'
  title: string
  detail?: string
  action?: string
  onAction?: () => void
}

export interface ListingPageProps<T> {
  title: string
  description?: string
  actions?: ReactNode
  metrics?: ListingMetric[]
  alerts?: ListingAlert[]
  filters?: { key: string; label: string; predicate: (row: T) => boolean }[]
  searchPlaceholder?: string
  querySyntax?: string
  searchIn?: (row: T) => string
  rows: T[]
  columns: Column<T>[]
  card: CardSpec<T>
  getKey: (row: T) => string
  getHref?: (row: T) => string
  aggregates?: Partial<Record<string, ReactNode>>
  emptyLabel?: string
  emptyHint?: string
  emptyAction?: ReactNode
  defaultView?: ViewMode
  loading?: boolean
}

const toneStyles = {
  ok: 'border-fk-success-border bg-fk-success-soft/50 text-fk-success',
  warn: 'border-fk-amber-border bg-fk-amber-soft/50 text-fk-amber',
  error: 'border-fk-danger-border bg-fk-danger-soft/40 text-fk-danger',
} as const

export function ListingPage<T>({
  title,
  description,
  actions,
  metrics,
  alerts,
  filters,
  searchPlaceholder = 'Search…',
  querySyntax,
  searchIn,
  rows,
  columns,
  card,
  getKey,
  getHref,
  aggregates,
  emptyLabel,
  emptyHint,
  emptyAction,
  defaultView = 'list',
  loading = false,
}: ListingPageProps<T>) {
  const [view, setView] = useState<ViewMode>(defaultView)
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<string>('all')

  const filtered = useMemo(() => {
    let out = rows
    const active = filters?.find((f) => f.key === filter)
    if (active) out = out.filter(active.predicate)
    if (query.trim() && searchIn) {
      const q = query.trim().toLowerCase()
      out = out.filter((r) => searchIn(r).toLowerCase().includes(q))
    }
    return out
  }, [rows, filters, filter, query, searchIn])

  const filteredOut = rows.length > 0 && filtered.length === 0

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <div
        className={cn(
          'flex flex-wrap items-start justify-between gap-3',
          tokens.header.spacing
        )}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{title}</h1>
          {/* Le style FerrisKey garde la description : il emprunte la densité
              du compact, pas son dépouillement — sur un listing, la ligne qui
              dit ce que la ressource est vaut la hauteur qu'elle coûte. */}
          {description && (
            <p className='mt-0.5 text-sm text-neutral-500'>{description}</p>
          )}
        </div>
        {actions && <div className='flex shrink-0 gap-2'>{actions}</div>}
      </div>

      <div className={tokens.page.sectionGap}>
        {/* FK-14 : ligne unique — le détail passe en suffixe grisé, l'action en
            lien. On garde l'information sans la hauteur d'un encart plein. */}
        {alerts && alerts.length > 0 && (
          <ul className='space-y-1'>
            {alerts.map((a) => (
              <li
                key={a.title}
                className={cn(
                  'flex items-center gap-2 rounded-sm border px-2.5 py-1.5 text-[13px]',
                  toneStyles[a.tone]
                )}
              >
                {a.tone === 'ok' ? (
                  <CheckCircle2 className='size-3.5 shrink-0' strokeWidth={2} />
                ) : (
                  <AlertTriangle className='size-3.5 shrink-0' strokeWidth={2} />
                )}
                <span className='shrink-0 font-medium text-neutral-900'>
                  {a.title}
                </span>
                {a.detail && (
                  <span className='min-w-0 truncate text-neutral-500'>
                    {a.detail}
                  </span>
                )}
                {a.action && (
                  <button
                    type='button'
                    onClick={a.onAction}
                    className='ml-auto shrink-0 cursor-pointer text-xs font-medium underline-offset-2 hover:underline'
                  >
                    {a.action} →
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}

        <MetricsBand metrics={metrics ?? []} />

        <div className='flex flex-wrap items-center gap-2'>
          <label className='relative flex h-8 min-w-[15rem] flex-1 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={
                tokens.toolbar.showQuerySyntax && querySyntax
                  ? querySyntax
                  : searchPlaceholder
              }
              className={cn(
                'h-full w-full rounded-md border border-fk-line bg-white pl-8 pr-3 outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15',
                tokens.toolbar.showQuerySyntax && querySyntax
                  ? 'font-mono-ui text-xs placeholder:text-neutral-300'
                  : 'text-sm'
              )}
            />
          </label>

          {filters && filters.length > 0 && (
            <div className='flex gap-1'>
              {[{ key: 'all', label: 'All' }, ...filters].map((f) => (
                <button
                  key={f.key}
                  type='button'
                  onClick={() => setFilter(f.key)}
                  className={cn(
                    'cursor-pointer rounded-md px-2.5 py-1.5 text-xs transition-colors',
                    f.key === filter
                      ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                      : 'text-neutral-500 hover:bg-neutral-100'
                  )}
                >
                  {f.label}
                </button>
              ))}
            </div>
          )}

          {tokens.toolbar.showViewToggle && (
            <div className='flex rounded-md border border-fk-line p-0.5'>
              {(
                [
                  ['list', List, 'List view'],
                  ['cards', LayoutGrid, 'Card view'],
                ] as const
              ).map(([mode, Icon, label]) => (
                <button
                  key={mode}
                  type='button'
                  onClick={() => setView(mode)}
                  aria-label={label}
                  aria-pressed={view === mode}
                  className={cn(
                    'grid size-6 cursor-pointer place-items-center rounded transition-colors',
                    view === mode
                      ? 'bg-fk-primary-soft text-fk-primary-text'
                      : 'text-neutral-400 hover:text-neutral-700'
                  )}
                >
                  <Icon className='size-3.5' />
                </button>
              ))}
            </div>
          )}
        </div>

        <DataView
          rows={filtered}
          columns={columns}
          card={card}
          getKey={getKey}
          getHref={getHref}
          view={view}
          loading={loading}
          aggregates={aggregates}
          emptyLabel={filteredOut ? 'No match' : emptyLabel}
          emptyHint={
            filteredOut
              ? `${rows.length} ${rows.length > 1 ? 'entries exist' : 'entry exists'} but ${rows.length > 1 ? 'are' : 'is'} hidden by the current filter.`
              : emptyHint
          }
          emptyAction={
            filteredOut ? (
              <div className='flex flex-wrap items-center justify-center gap-2'>
                <Button
                  variant='outline'
                  onClick={() => {
                    setQuery('')
                    setFilter('all')
                  }}
                >
                  Clear filter
                </Button>
                {emptyAction}
              </div>
            ) : (
              emptyAction
            )
          }
        />

        {!loading && rows.length > 0 && (
          <p className='tnum text-xs text-neutral-400'>
            {filtered.length} of {rows.length}
          </p>
        )}
      </div>
    </div>
  )
}
