import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { MetricsBand } from '@/components/kit'
import type { Metric } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { formatTimestamp } from '@/utils/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import { journalFilterFields } from '@/pages/iam/seawatch/event-filter-fields'
import type { EventJournal } from '../feature/use-event-journal'
import { eventCard, eventColumns } from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

export interface SessionCounts {
  opened: number
  revoked: number
  total: number
}

export interface PageSessionsProps {
  events: SecurityEvent[]
  counts: SessionCounts
  journal: EventJournal
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
  directory: RealmDirectory
}

export default function PageSessions({
  events,
  counts,
  journal,
  isLoading,
  isError,
  truncated,
  windowDays,
  windowLimit,
  directory,
}: PageSessionsProps) {
  const { t } = useTranslation(CONSOLE_NAMESPACES)

  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const tabs = [
    { value: '', label: t('activity.filter.all') },
    { value: 'session_created', label: t('activity.sessions.filters.opened') },
    { value: 'session_revoked', label: t('activity.sessions.filters.revoked') },
  ]

  const opened = events.filter((e) => e.event_type === 'session_created')
  const accounts = new Set(events.map((e) => e.actor_id ?? 'unknown')).size
  const origins = new Set(events.map((e) => e.ip_address).filter(Boolean)).size

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: number[]) => (!truncated && series.length > 0 ? series : undefined)
  const counted = (series: number[], value: number) =>
    !truncated && series.length > 0 ? series : [value, value]

  const metrics: Metric[] = [
    {
      key: 'opened',
      label: t('activity.sessions.metrics.opened'),
      value: t('number', { value: counts.opened }),
      hint: overWindow,
      series: counted(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_created').length),
        counts.opened
      ),
      tone: 'success',
    },
    {
      key: 'revoked',
      label: t('activity.sessions.metrics.revoked'),
      value: t('number', { value: counts.revoked }),
      hint: counts.revoked === 0 ? t('activity.sessions.metrics.none_revoked') : overWindow,
      series: counted(
        buckets.map((b) => b.filter((e) => e.event_type === 'session_revoked').length),
        counts.revoked
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
  const journalColumns = [
    { ...columns.event, sortKey: 'event_type' },
    { ...columns.outcome, sortKey: 'status' },
    columns.actor,
    columns.origin,
    { ...columns.when, sortKey: 'timestamp' },
  ]

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
        rows={journal.events}
        pagination={journal.pagination}
        listing={journal.listing}
        tabs={tabs}
        filterFields={journalFilterFields()}
        columns={journalColumns}
        card={eventCard(directory, t)}
        getKey={(e) => e.id}
        loading={isLoading || journal.isLoading}
        aggregates={{
          event_type: t('activity.sessions.aggregates.opened', { total: journal.firstCount }),
          status: t('activity.sessions.aggregates.revoked', { total: journal.secondCount }),
        }}
        emptyLabel={t('activity.sessions.journal.empty_label')}
        emptyHint={t('activity.sessions.journal.empty_hint')}
      />
    </ActivityPage>
  )
}
