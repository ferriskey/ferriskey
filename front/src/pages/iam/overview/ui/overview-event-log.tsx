import { useTranslation } from 'react-i18next'
import { Pill, type PillTone } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatDateTime as formatTime } from '@/utils/format-date'

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

const eventStatusLabelKey = (status: OverviewEvent['status']) => `event_status.${status}`

const SECOND_IN_MS = 1000

export default function OverviewEventLog({ events, emptyLabel }: OverviewEventLogProps) {
  const { t } = useTranslation('overview')

  if (events.length === 0) {
    return (
      <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-10')}>
        <p className='text-sm text-neutral-500 dark:text-neutral-400'>{emptyLabel}</p>
      </div>
    )
  }

  const formatDuration = (ms?: number | null) => {
    if (ms === undefined || ms === null) return null
    return ms < SECOND_IN_MS
      ? t('events.duration.milliseconds', { value: ms })
      : t('events.duration.seconds', { value: (ms / SECOND_IN_MS).toFixed(1) })
  }

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
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>
              {t('events.columns.status')}
            </th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>
              {t('events.columns.grant')}
            </th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>
              {t('events.columns.client')}
            </th>
            <th className={cn(tokens.table.headerPadding, 'font-medium')}>
              {t('events.columns.account')}
            </th>
            <th className={cn(tokens.table.headerPadding, 'text-right font-medium')}>
              {t('events.columns.duration')}
            </th>
            <th className={cn(tokens.table.headerPadding, 'text-right font-medium')}>
              {t('events.columns.started')}
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
                  'transition-colors hover:bg-neutral-50 dark:hover:bg-fk-surface',
                  event.status === 'failure' && 'bg-fk-danger-soft/25'
                )}
              >
                <td className={tokens.table.cellPadding}>
                  <Pill tone={statusTone[event.status]} mono>
                    {t(eventStatusLabelKey(event.status))}
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
                    <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                      {t('events.no_client')}
                    </span>
                  )}
                </td>
                <td className={tokens.table.cellPadding}>
                  {event.user ? (
                    <span className='text-neutral-900 dark:text-neutral-100'>{event.user}</span>
                  ) : (
                    <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                      {t('events.not_identified')}
                    </span>
                  )}
                </td>
                <td className={cn(tokens.table.cellPadding, 'tnum text-right text-neutral-600 dark:text-neutral-400')}>
                  {duration ?? (
                    <span className='text-neutral-300 dark:text-neutral-600'>
                      {t('events.not_measured')}
                    </span>
                  )}
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
