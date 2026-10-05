import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertTriangle, CheckCircle2, LayoutGrid, List, X } from 'lucide-react'
import { Button } from '@/components/kit/button'
import type { ChartTone } from './charts'
import { MetricsBand } from './MetricsBand'
import { DataView, type CardSpec, type Column, type ViewMode } from './DataView'
import { PageShell } from './page-shell'
import { FilterBar, type FilterField } from './FilterBar'
import { clearColumnFilters, countActiveFilters } from './column-filter-state'
import { SearchInput } from './SearchInput'
import { PaginationBar } from './PaginationBar'
import type { PaginationMetadata } from './listing-query-state'
import type { PagedListing } from './use-paged-listing'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { useLayoutTier } from '@/hooks/use-media-query'

export interface ListingMetric {
  key: string
  label: string
  value: number | string
  hint?: string
  delta?: number
  series?: (number | null)[]
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
  insights?: ReactNode
  paged: {
    listing: PagedListing
    pagination: PaginationMetadata | undefined
    search?: { placeholder: string }
    filterFields?: FilterField[]
  }
  searchScopeHint?: string
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

const SEARCH_KEY = 'search'

const VIEW_MODES = [
  ['list', List],
  ['cards', LayoutGrid],
] as const

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
  insights,
  paged,
  searchScopeHint,
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
  const { t } = useTranslation()
  const [view, setView] = useState<ViewMode>(defaultView)
  const tier = useLayoutTier()
  const effectiveView = tier === 'phone' ? 'cards' : view

  const narrowed = Object.values(paged.listing.state.filters).some(Boolean)
  const filteredOut = narrowed && rows.length === 0

  const activeFilters = countActiveFilters(columns, paged.listing.state.filters)
  const clearColumns = () =>
    paged.listing.setFilters(clearColumnFilters(columns.flatMap((col) => col.filters ?? [])))

  const entryCount =
    paged.pagination && t('data_view.entry_count', { count: paged.pagination.total })

  return (
    <PageShell>
      <div
        className={cn(
          'flex flex-wrap items-start justify-between gap-3',
          tokens.header.spacing
        )}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>{title}</h1>
          {description && (
            <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>{description}</p>
          )}
        </div>
        {actions && <div className='flex shrink-0 gap-2'>{actions}</div>}
      </div>

      <div className={tokens.page.sectionGap}>
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
                <span className='shrink-0 font-medium text-neutral-900 dark:text-neutral-100'>
                  {a.title}
                </span>
                {a.detail && (
                  <span className='min-w-0 truncate text-neutral-500 dark:text-neutral-400'>
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

        {insights}

        <div className='flex flex-wrap items-center gap-2'>
          {paged.filterFields && (
            <FilterBar fields={paged.filterFields} listing={paged.listing} />
          )}

          {paged.search && (
            <SearchInput
              value={paged.listing.drafts[SEARCH_KEY] ?? ''}
              onChange={(value) => paged.listing.setDraft(SEARCH_KEY, value)}
              placeholder={paged.search.placeholder}
            />
          )}

          {activeFilters > 0 && (
            <Button variant='ghost' size='sm' onClick={clearColumns}>
              <X />
              {t('listing.clear_filters', { count: activeFilters })}
            </Button>
          )}

          {!paged.filterFields && !paged.search && <div className='flex-1' />}

          {tokens.toolbar.showViewToggle && tier !== 'phone' && (
            <div className='flex rounded-md border border-fk-line p-0.5'>
              {VIEW_MODES.map(([mode, Icon]) => (
                <button
                  key={mode}
                  type='button'
                  onClick={() => setView(mode)}
                  aria-label={t(`data_view.view_mode.${mode}`)}
                  aria-pressed={view === mode}
                  className={cn(
                    'grid size-6 cursor-pointer place-items-center rounded transition-colors',
                    view === mode
                      ? 'bg-fk-primary-soft text-fk-primary-text'
                      : 'text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300'
                  )}
                >
                  <Icon className='size-3.5' />
                </button>
              ))}
            </div>
          )}
        </div>

        {searchScopeHint && (
          <p className='-mt-1 text-xs text-neutral-400 dark:text-neutral-500'>{searchScopeHint}</p>
        )}

        <DataView
          listing={paged.listing}
          rows={rows}
          columns={columns}
          card={card}
          getKey={getKey}
          getHref={getHref}
          view={effectiveView}
          loading={loading}
          aggregates={aggregates}
          sort={paged.listing.state.sort}
          onSortChange={paged.listing.setSort}
          emptyLabel={filteredOut ? t('data_view.no_match') : emptyLabel}
          emptyHint={filteredOut ? t('data_view.no_match_server_hint') : emptyHint}
          emptyAction={
            filteredOut ? (
              <div className='flex flex-wrap items-center justify-center gap-2'>
                <Button variant='outline' onClick={() => paged.listing.clearFilters()}>
                  {t('data_view.clear_filter')}
                </Button>
                {emptyAction}
              </div>
            ) : (
              emptyAction
            )
          }
        />

        {paged.pagination && (
          <PaginationBar pagination={paged.pagination} onPageChange={paged.listing.setPage} />
        )}

        {!loading && rows.length > 0 && entryCount && (
          <p className='tnum text-xs text-neutral-400 dark:text-neutral-500'>{entryCount}</p>
        )}
      </div>
    </PageShell>
  )
}
