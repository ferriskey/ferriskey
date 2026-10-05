import type { KeyboardEvent, ReactNode } from 'react'
import { ArrowUpRight } from 'lucide-react'
import { Sparkline, type ChartTone } from './charts'
import { isTileSelected, tileFilterPatch, type TileFilter } from './metric-tile-filter'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface Metric {
  key: string
  label: string
  value: number | string
  hint?: string
  delta?: number
  series?: (number | null)[]
  tone?: ChartTone
  filter?: TileFilter
}

export interface MetricsBandFilter {
  filters: Record<string, string>
  onChange: (patch: Record<string, string>) => void
}

const SELECT_KEYS = ['Enter', ' ']

function Tile({
  metric,
  metrics,
  filtering,
  className,
  children,
}: {
  metric: Metric
  metrics: Metric[]
  filtering?: MetricsBandFilter
  className: string
  children: ReactNode
}) {
  const { filter } = metric
  if (!filter || !filtering) return <div className={className}>{children}</div>
  const selected = isTileSelected(metrics, filter, filtering.filters)
  const select = () => filtering.onChange(tileFilterPatch(metrics, filter))
  return (
    <div
      role='button'
      tabIndex={0}
      aria-pressed={selected}
      onClick={select}
      onKeyDown={(event: KeyboardEvent<HTMLDivElement>) => {
        if (!SELECT_KEYS.includes(event.key)) return
        event.preventDefault()
        select()
      }}
      className={cn(
        className,
        'cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-fk-primary/30',
        selected
          ? 'bg-fk-primary-soft ring-1 ring-inset ring-fk-primary-border'
          : 'hover:bg-fk-primary-soft/40'
      )}
    >
      {children}
    </div>
  )
}

export function MetricsBand({
  metrics,
  filtering,
}: {
  metrics: Metric[]
  filtering?: MetricsBandFilter
}) {
  if (metrics.length === 0) return null

  const hasSeries = metrics.some((m) => m.series)

  return (
    <div
      className={cn(
        tokens.surface.panel,
        'grid grid-cols-2 divide-fk-line overflow-hidden lg:grid-cols-4 lg:divide-x'
      )}
    >
      {metrics.map((m) =>
        hasSeries ? (
          <Tile key={m.key} metric={m} metrics={metrics} filtering={filtering} className='flex items-center gap-3 px-3 py-2'>
            <div className='min-w-0 flex-1'>
              <p className='truncate text-[11px] text-neutral-500 dark:text-neutral-400'>{m.label}</p>
              <div className='flex items-baseline gap-1.5'>
                <span className='tnum text-lg font-semibold leading-tight'>
                  {m.value}
                </span>
                {m.delta !== undefined && m.delta !== 0 && (
                  <span className='tnum inline-flex items-center text-[11px] text-fk-success'>
                    <ArrowUpRight className='size-3' />
                    {m.delta}
                  </span>
                )}
                {m.hint && (
                  <span className='truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {m.hint}
                  </span>
                )}
              </div>
            </div>
            {m.series && (
              <div className='w-16 shrink-0'>
                <Sparkline data={m.series} tone={m.tone ?? 'info'} height={28} />
              </div>
            )}
          </Tile>
        ) : (
          <Tile key={m.key} metric={m} metrics={metrics} filtering={filtering} className='px-3 py-2'>
            <p className='truncate text-[11px] text-neutral-500 dark:text-neutral-400'>{m.label}</p>
            <div className='mt-0.5 flex items-baseline gap-1.5'>
              <span className='tnum text-xl font-semibold leading-none'>
                {m.value}
              </span>
              {m.hint && (
                <span className='truncate text-[11px] text-neutral-400 dark:text-neutral-500'>
                  {m.hint}
                </span>
              )}
            </div>
          </Tile>
        )
      )}
    </div>
  )
}
