import { Pill, type PillTone } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDateTime as formatTime } from '@/next/shared/format-date'

export interface OverviewEvent {
  id: string
  status: 'pending' | 'success' | 'failure' | 'expired'
  grantType: string
  client?: string
  user?: string
  startedAt: string
  durationMs?: number | null
}

export interface OverviewEventLogProps {
  events: OverviewEvent[]
  emptyLabel: string
}

const statusTone: Record<OverviewEvent['status'], PillTone> = {
  success: 'success',
  failure: 'danger',
  pending: 'info',
  expired: 'amber',
}

const formatDuration = (ms?: number | null) => {
  if (ms === undefined || ms === null) return null
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s`
}

export default function OverviewEventLog({ events, emptyLabel }: OverviewEventLogProps) {
  if (events.length === 0) {
    return (
      <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-10')}>
        <p className='text-sm text-neutral-500 dark:text-neutral-400'>{emptyLabel}</p>
      </div>
    )
  }

  return (
    <div className={cn(tokens.surface.panel, 'overflow-x-auto')}>
      <table className={cn('w-full', tokens.table.text)}>
        <thead>
          <tr
            className={cn(
              'border-b border-fk-line bg-neutral-50/60 dark:bg-neutral-900/60 text-left',
              tokens.table.headerText
            )}
          >
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>Status</th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>Grant</th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>Client</th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>Account</th>
            <th className={cn(tokens.table.headerPadding, 'text-right font-medium')}>
              Duration
            </th>
            <th className={cn(tokens.table.headerPadding, 'text-right font-medium')}>
              Started
            </th>
          </tr>
        </thead>
        <tbody className={tokens.surface.divider}>
          {events.map((event) => {
            const duration = formatDuration(event.durationMs)
            return (
              <tr
                key={event.id}
                className={cn(
                  'transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900',
                  event.status === 'failure' && 'bg-fk-danger-soft/25'
                )}
              >
                <td className={tokens.table.cellPadding}>
                  <Pill tone={statusTone[event.status]} mono>
                    {event.status}
                  </Pill>
                </td>
                <td className={cn(tokens.table.cellPadding, 'font-mono-ui text-[11px] text-neutral-600 dark:text-neutral-400')}>
                  {event.grantType}
                </td>
                <td className={tokens.table.cellPadding}>
                  {event.client ? (
                    <span className='font-mono-ui text-[11px] text-neutral-600 dark:text-neutral-400'>
                      {event.client}
                    </span>
                  ) : (
                    <span className='text-xs text-neutral-400 dark:text-neutral-500'>no client</span>
                  )}
                </td>
                <td className={tokens.table.cellPadding}>
                  {event.user ? (
                    <span className='text-neutral-900 dark:text-neutral-100'>{event.user}</span>
                  ) : (
                    <span className='text-xs text-neutral-400 dark:text-neutral-500'>not identified</span>
                  )}
                </td>
                <td className={cn(tokens.table.cellPadding, 'tnum text-right text-neutral-600 dark:text-neutral-400')}>
                  {duration ?? <span className='text-neutral-300 dark:text-neutral-600'>not measured</span>}
                </td>
                <td
                  className={cn(
                    tokens.table.cellPadding,
                    'tnum text-right text-[11px] text-neutral-400 dark:text-neutral-500'
                  )}
                >
                  {formatTime(event.startedAt)}
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}
