import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { LayoutGrid, List } from 'lucide-react'
import { Button, DataView, FilterBar, PaginationBar, Section } from '@/components/kit'
import type {
  CardSpec,
  Column,
  FilterField,
  PagedListing,
  PaginationMetadata,
  ViewMode,
} from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { useLayoutTier } from '@/hooks/use-media-query'

const TAB_FILTER_KEY = 'event_types'

const VIEW_TOGGLES = [
  { mode: 'list', icon: List, labelKey: 'activity.journal.view.list' },
  { mode: 'cards', icon: LayoutGrid, labelKey: 'activity.journal.view.cards' },
] as const

export interface JournalTab {
  value: string
  label: string
}

export interface JournalSectionProps<T> {
  title: string
  description: string
  rows: T[]
  pagination: PaginationMetadata | undefined
  listing: PagedListing
  tabs: JournalTab[]
  filterFields: FilterField[]
  columns: Column<T>[]
  card: CardSpec<T>
  getKey: (row: T) => string
  loading: boolean
  aggregates?: Partial<Record<string, ReactNode>>
  emptyLabel: string
  emptyHint: string
}

export function JournalSection<T>({
  title,
  description,
  rows,
  pagination,
  listing,
  tabs,
  filterFields,
  columns,
  card,
  getKey,
  loading,
  aggregates,
  emptyLabel,
  emptyHint,
}: JournalSectionProps<T>) {
  const { t } = useTranslation('console')
  const [view, setView] = useState<ViewMode>('list')

  const tier = useLayoutTier()
  const effectiveView = tier === 'phone' ? 'cards' : view

  const activeTab = listing.state.filters[TAB_FILTER_KEY] ?? ''
  const narrowed = Object.values(listing.state.filters).some(Boolean)
  const filteredOut = narrowed && rows.length === 0

  return (
    <Section
      title={title}
      description={description}
      contained={false}
      action={
        <div className='flex flex-wrap items-center justify-end gap-2'>
          <div className='flex gap-1'>
            {tabs.map((tab) => (
              <button
                key={tab.value}
                type='button'
                onClick={() => listing.setFilter(TAB_FILTER_KEY, tab.value)}
                className={cn(
                  'cursor-pointer rounded-md px-2 py-1 text-xs transition-colors',
                  tab.value === activeTab
                    ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                    : 'text-neutral-500 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-fk-raised'
                )}
              >
                {tab.label}
              </button>
            ))}
          </div>
          {tokens.toolbar.showViewToggle && tier !== 'phone' && (
            <div className='flex rounded-md border border-fk-line p-0.5'>
              {VIEW_TOGGLES.map(({ mode, icon: Icon, labelKey }) => (
                <button
                  key={mode}
                  type='button'
                  onClick={() => setView(mode)}
                  aria-label={t(labelKey)}
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
      }
    >
      <div className={tokens.page.sectionGap}>
        <div className='flex flex-wrap items-center gap-2'>
          <FilterBar fields={filterFields} listing={listing} />
        </div>

        <DataView
          rows={rows}
          columns={columns}
          card={card}
          getKey={getKey}
          view={effectiveView}
          loading={loading}
          aggregates={aggregates}
          sort={listing.state.sort}
          onSortChange={listing.setSort}
          emptyLabel={filteredOut ? t('activity.journal.filtered.label') : emptyLabel}
          emptyHint={filteredOut ? t('activity.journal.filtered.hint') : emptyHint}
          emptyAction={
            filteredOut ? (
              <Button variant='outline' onClick={() => listing.clearFilters()}>
                {t('activity.journal.filtered.action')}
              </Button>
            ) : undefined
          }
        />

        {pagination && (
          <PaginationBar pagination={pagination} onPageChange={listing.setPage} />
        )}

        {!loading && pagination && pagination.total > 0 && (
          <p className='tnum text-xs text-neutral-400 dark:text-neutral-500'>
            {t('activity.journal.count', { count: pagination.total })}
          </p>
        )}
      </div>
    </Section>
  )
}
