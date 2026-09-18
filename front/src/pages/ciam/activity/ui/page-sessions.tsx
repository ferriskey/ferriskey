import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { MetricsBand } from '@/components/kit'
import type { Metric } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { formatTimestamp } from '@/utils/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import { eventCard, eventColumns, searchEvent } from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

export interface PageSessionsProps {
  events: SecurityEvent[]
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
  directory: RealmDirectory
}

export default function PageSessions({
  events,
  isLoading,
  isError,
  truncated,
  windowDays,
  windowLimit,
  directory,
}: PageSessionsProps) {
  const { t } = useTranslation(CONSOLE_NAMESPACES)
  const [filter, setFilter] = useState('all')
  const [query, setQuery] = useState('')

  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const filters = useMemo(
    () => [
      { key: 'all', label: t('activity.filter.all') },
      { key: 'opened', label: t('activity.sessions.filters.opened') },
      { key: 'revoked', label: t('activity.sessions.filters.revoked') },
    ],
    [t]
  )

  const opened = events.filter((e) => e.event_type === 'session_created')
  const revoked = events.filter((e) => e.event_type === 'session_revoked')
  const accounts = new Set(events.map((e) => e.actor_id ?? 'unknown')).size
  const origins = new Set(events.map((e) => e.ip_address).filter(Boolean)).size

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: number[]) => (!truncated && series.length > 0 ? series : undefined)

  const metrics: Metric[] = [
    {
      key: 'opened',
      label: t('activity.sessions.metrics.opened'),
      value: t('number', { value: opened.length }),
      hint: overWindow,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_created').length)
      ),
      tone: 'success',
    },
    {
      key: 'revoked',
      label: t('activity.sessions.metrics.revoked'),
      value: t('number', { value: revoked.length }),
      hint: revoked.length === 0 ? t('activity.sessions.metrics.none_revoked') : overWindow,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_revoked').length)
      ),
      tone: 'brand',
    },
    {
      key: 'accounts',
      label: t('activity.sessions.metrics.accounts'),
      value: t('number', { value: accounts }),
      hint: overWindow,
      series: measured(
        buckets.map((b) => new Set(b.map((e) => e.actor_id ?? 'unknown')).size)
      ),
      tone: 'violet',
    },
    {
      key: 'origins',
      label: t('activity.sessions.metrics.origins'),
      value: t('number', { value: origins }),
      hint:
        origins === 0
          ? t('activity.sessions.metrics.no_origin')
          : t('activity.sessions.metrics.origins_hint'),
      series: measured(
        buckets.map((b) => new Set(b.map((e) => e.ip_address).filter(Boolean)).size)
      ),
      tone: 'info',
    },
  ]

  const notices: Notice[] = [
    {
      tone: 'note',
      title: t('activity.sessions.notices.no_device_list.title'),
      detail: t('activity.sessions.notices.no_device_list.detail'),
    },
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('activity.sessions.notices.error.title'),
            detail: t('activity.sessions.notices.error.detail'),
          },
        ]
      : []),
    ...(truncated
      ? [
          {
            tone: 'warn' as const,
            title: t('activity.capped.title', { limit: windowLimit }),
            detail: t('activity.capped.detail_above', {
              window: windowLabel,
              limit: windowLimit,
            }),
          },
        ]
      : []),
  ]

  const columns = eventColumns(directory, t)

  const filtered = useMemo(() => {
    const byFilter =
      filter === 'opened'
        ? events.filter((e) => e.event_type === 'session_created')
        : filter === 'revoked'
          ? events.filter((e) => e.event_type === 'session_revoked')
          : events
    const needle = query.trim().toLowerCase()
    if (!needle) return byFilter
    return byFilter.filter((e) => searchEvent(e).toLowerCase().includes(needle))
  }, [events, filter, query])

  const lastOpened = opened[0]

  return (
    <ActivityPage
      title={t('activity.sessions.title')}
      description={
        lastOpened
          ? t('activity.sessions.description_with_latest', {
              window: windowLabel,
              timestamp: formatTimestamp(lastOpened.timestamp),
            })
          : t('activity.sessions.description', { window: windowLabel })
      }
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title={t('activity.sessions.journal.title')}
        description={t('activity.sessions.journal.description')}
        rows={filtered}
        total={events.length}
        columns={[
          columns.event,
          columns.outcome,
          columns.actor,
          columns.origin,
          columns.when,
        ]}
        card={eventCard(directory, t)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={filter}
        onFilter={setFilter}
        query={query}
        onQuery={setQuery}
        searchPlaceholder={t('activity.sessions.journal.search_placeholder')}
        aggregates={{
          event_type: t('activity.sessions.aggregates.opened', { total: opened.length }),
          status: t('activity.sessions.aggregates.revoked', { total: revoked.length }),
          actor: t('activity.sessions.aggregates.accounts', { count: accounts }),
        }}
        emptyLabel={t('activity.sessions.journal.empty_label')}
        emptyHint={t('activity.sessions.journal.empty_hint', { window: windowLabel })}
      />
    </ActivityPage>
  )
}
