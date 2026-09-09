import { useMemo, useState, type ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { ArrowUpDown } from 'lucide-react'
import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { EmptyState } from './empty-state'

export type ViewMode = 'list' | 'cards'

export interface Column<T> {
  key: string
  header: string
  render: (row: T) => ReactNode
  sortValue?: (row: T) => string | number
  align?: 'right'
  headerClassName?: string
  cellClassName?: string
}

export interface CardSpec<T> {
  avatar?: (row: T) => ReactNode
  title: (row: T) => ReactNode
  subtitle?: (row: T) => ReactNode
  badges?: (row: T) => ReactNode
  flags?: (row: T) => { label: string; on: boolean }[]
  footer?: (row: T) => ReactNode
}

export interface DataViewProps<T> {
  rows: T[]
  columns: Column<T>[]
  card: CardSpec<T>
  getKey: (row: T) => string
  getHref?: (row: T) => string
  view: ViewMode
  aggregates?: Partial<Record<string, ReactNode>>
  emptyLabel?: string
  emptyHint?: string
  emptyAction?: ReactNode
  loading?: boolean
}

export function DataView<T>({
  rows,
  columns,
  card,
  getKey,
  getHref,
  view,
  aggregates,
  emptyLabel = 'No results',
  emptyHint,
  emptyAction,
  loading = false,
}: DataViewProps<T>) {
  const [sort, setSort] = useState<{ key: string; dir: 1 | -1 } | null>(null)
  const [selected, setSelected] = useState<string[]>([])

  const sorted = useMemo(() => {
    if (!sort) return rows
    const col = columns.find((c) => c.key === sort.key)
    if (!col?.sortValue) return rows
    const value = col.sortValue
    return [...rows].sort((a, b) => {
      const va = value(a)
      const vb = value(b)
      if (typeof va === 'number' && typeof vb === 'number')
        return (va - vb) * sort.dir
      return String(va).localeCompare(String(vb)) * sort.dir
    })
  }, [rows, sort, columns])

  if (loading) {
    return (
      <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className='flex items-center gap-3 px-2.5 py-3'>
            <div className='h-3 w-1/4 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-3 w-1/6 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-3 w-1/5 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        ))}
      </div>
    )
  }

  if (rows.length === 0) {
    return <EmptyState label={emptyLabel} hint={emptyHint} action={emptyAction} />
  }

  if (view === 'cards') {
    return (
      <div className={cn('grid grid-cols-1', tokens.card.gap, tokens.card.columns)}>
        {sorted.map((row) => {
          const href = getHref?.(row)
          const flags = tokens.card.showFlows ? card.flags?.(row) : undefined

          const body = (
            <>
              <div className='flex items-start gap-3'>
                {card.avatar?.(row)}
                <div className='min-w-0 flex-1'>
                  <div className='truncate font-medium text-neutral-900 dark:text-neutral-100 transition-colors group-hover:text-fk-primary-text'>
                    {card.title(row)}
                  </div>
                  {card.subtitle && (
                    <div className='truncate font-mono-ui text-xs text-neutral-400 dark:text-neutral-500'>
                      {card.subtitle(row)}
                    </div>
                  )}
                </div>
              </div>

              {card.badges && (
                <div className='mt-3 flex flex-wrap gap-1.5'>{card.badges(row)}</div>
              )}

              {flags && flags.length > 0 && (
                <ul className='mt-3 space-y-1.5 border-t border-fk-line-soft pt-3'>
                  {flags.map((f) => (
                    <li key={f.label} className='flex items-center gap-2 text-xs'>
                      <span
                        className={cn(
                          'size-1.5 shrink-0 rounded-full',
                          f.on ? 'bg-fk-success' : 'bg-neutral-200 dark:bg-fk-raised'
                        )}
                      />
                      <span className={
                          f.on
                            ? 'text-neutral-700 dark:text-neutral-300'
                            : 'text-neutral-400 dark:text-neutral-500'
                        }>
                        {f.label}
                      </span>
                    </li>
                  ))}
                </ul>
              )}

              {card.footer && (
                <div className='mt-3 flex items-center justify-between border-t border-fk-line-soft pt-3 text-xs text-neutral-500 dark:text-neutral-400'>
                  {card.footer(row)}
                </div>
              )}
            </>
          )

          const className = cn(
            tokens.surface.panel,
            tokens.card.padding,
            'group flex flex-col transition-all',
            href &&
              'hover:border-fk-primary-border hover:shadow-[0_2px_12px_rgb(0_0_0/0.05)]'
          )

          return href ? (
            <Link key={getKey(row)} to={href} className={className}>
              {body}
            </Link>
          ) : (
            <div key={getKey(row)} className={className}>
              {body}
            </div>
          )
        })}
      </div>
    )
  }

  const selectable = tokens.table.selectable
  const allSelected = selected.length === rows.length
  const toggleAll = () => setSelected(allSelected ? [] : rows.map((r) => getKey(r)))

  return (
    <div className={cn(tokens.surface.panel, 'overflow-x-auto')}>
      <table className={cn('w-full', tokens.table.text)}>
        <thead>
          <tr
            className={cn(
              'border-b border-fk-line bg-neutral-50/60 dark:bg-fk-surface/60 text-left',
              tokens.table.headerText
            )}
          >
            {selectable && (
              <th className={cn('w-10', tokens.table.headerPadding)}>
                <Checkbox
                  checked={allSelected}
                  onCheckedChange={toggleAll}
                  aria-label='Select all'
                />
              </th>
            )}
            {columns.map((col) => (
              <th
                key={col.key}
                className={cn(
                  tokens.table.headerPadding,
                  'font-medium',
                  col.align === 'right' && 'text-right',
                  col.headerClassName
                )}
              >
                {tokens.table.sortable && col.sortValue ? (
                  <button
                    type='button'
                    onClick={() =>
                      setSort((s) =>
                        s?.key === col.key
                          ? { key: col.key, dir: s.dir === 1 ? -1 : 1 }
                          : { key: col.key, dir: 1 }
                      )
                    }
                    className={cn(
                      'inline-flex cursor-pointer items-center gap-1 transition-colors hover:text-neutral-700 dark:hover:text-neutral-300',
                      sort?.key === col.key && 'text-fk-primary-text'
                    )}
                  >
                    {col.header}
                    <ArrowUpDown className='size-3' />
                  </button>
                ) : (
                  col.header
                )}
              </th>
            ))}
          </tr>
        </thead>

        <tbody className={tokens.surface.divider}>
          {sorted.map((row) => {
            const key = getKey(row)
            const href = getHref?.(row)
            return (
              <tr
                key={key}
                className={cn(
                  'transition-colors hover:bg-neutral-50 dark:hover:bg-fk-surface',
                  selected.includes(key) && 'bg-fk-primary-soft/40'
                )}
              >
                {selectable && (
                  <td className={tokens.table.cellPadding}>
                    <Checkbox
                      checked={selected.includes(key)}
                      onCheckedChange={() =>
                        setSelected((s) =>
                          s.includes(key) ? s.filter((x) => x !== key) : [...s, key]
                        )
                      }
                      aria-label={`Select ${key}`}
                    />
                  </td>
                )}
                {columns.map((col, ci) => {
                  const content = col.render(row)
                  return (
                    <td
                      key={col.key}
                      className={cn(
                        tokens.table.cellPadding,
                        col.align === 'right' && 'text-right',
                        col.cellClassName
                      )}
                    >
                      {ci === 0 && href ? (
                        <Link
                          to={href}
                          className='font-medium text-neutral-900 dark:text-neutral-100 transition-colors hover:text-fk-primary-text'
                        >
                          {content}
                        </Link>
                      ) : (
                        content
                      )}
                    </td>
                  )
                })}
              </tr>
            )
          })}
        </tbody>

        {tokens.table.showFooterAggregates && aggregates && (
          <tfoot>
            <tr className='border-t border-fk-line bg-neutral-50/60 text-xs text-neutral-500 dark:bg-fk-surface/60 dark:text-neutral-400'>
              {selectable && <td className={tokens.table.cellPadding} />}
              {columns.map((col) => (
                <td
                  key={col.key}
                  className={cn(
                    tokens.table.cellPadding,
                    'font-medium',
                    col.align === 'right' && 'text-right'
                  )}
                >
                  {aggregates[col.key] ?? null}
                </td>
              ))}
            </tr>
          </tfoot>
        )}
      </table>
    </div>
  )
}
