import { useState, type ReactNode } from 'react'
import { LayoutGrid, List, Search } from 'lucide-react'
import { Button, DataView, Section } from '@/components/kit'
import type { CardSpec, Column, ViewMode } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface JournalFilter {
  key: string
  label: string
}

export interface JournalSectionProps<T> {
  title: string
  description: string
  rows: T[]
  total: number
  columns: Column<T>[]
  card: CardSpec<T>
  getKey: (row: T) => string
  loading: boolean
  filters?: JournalFilter[]
  filter?: string
  onFilter?: (key: string) => void
  query: string
  onQuery: (value: string) => void
  searchPlaceholder: string
  aggregates?: Partial<Record<string, ReactNode>>
  emptyLabel: string
  emptyHint: string
}

export function JournalSection<T>({
  title,
  description,
  rows,
  total,
  columns,
  card,
  getKey,
  loading,
  filters,
  filter = 'all',
  onFilter,
  query,
  onQuery,
  searchPlaceholder,
  aggregates,
  emptyLabel,
  emptyHint,
}: JournalSectionProps<T>) {
  const [view, setView] = useState<ViewMode>('list')

  const narrowed = Boolean(query.trim()) || filter !== 'all'
  const filteredOut = narrowed && rows.length === 0

  return (
    <Section
      title={title}
      description={description}
      contained={false}
      action={
        <div className='flex flex-wrap items-center justify-end gap-2'>
          {filters && onFilter && (
            <div className='flex gap-1'>
              {filters.map((f) => (
                <button
                  key={f.key}
                  type='button'
                  onClick={() => onFilter(f.key)}
                  className={cn(
                    'cursor-pointer rounded-md px-2 py-1 text-xs transition-colors',
                    f.key === filter
                      ? 'bg-fk-primary-soft font-medium text-fk-primary-text'
                      : 'text-neutral-500 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-fk-raised'
                  )}
                >
                  {f.label}
                </button>
              ))}
            </div>
          )}
          <label className='relative flex h-7 w-56 items-center'>
            <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(e) => onQuery(e.target.value)}
              placeholder={searchPlaceholder}
              className='h-full w-full rounded-md border border-fk-line bg-white pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15 dark:bg-fk-surface'
            />
          </label>
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
        <DataView
          rows={rows}
          columns={columns}
          card={card}
          getKey={getKey}
          view={view}
          loading={loading}
          aggregates={aggregates}
          emptyLabel={filteredOut ? 'No match' : emptyLabel}
          emptyHint={
            filteredOut
              ? 'No entry matches the current search and filter.'
              : emptyHint
          }
          emptyAction={
            filteredOut ? (
              <Button
                variant='outline'
                onClick={() => {
                  onQuery('')
                  onFilter?.('all')
                }}
              >
                Clear filter
              </Button>
            ) : undefined
          }
        />

        {!loading && total > 0 && (
          <p className='tnum text-xs text-neutral-400 dark:text-neutral-500'>
            {rows.length === total
              ? `${total} ${total > 1 ? 'entries' : 'entry'}`
              : `${rows.length} of ${total}`}
          </p>
        )}
      </div>
    </Section>
  )
}
