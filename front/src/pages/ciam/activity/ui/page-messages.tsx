import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { MetricsBand, Pill, Section } from '@/components/kit'
import type { Column, Metric } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/hooks/use-realm-directory'
import { formatRelative, formatTimestamp } from '@/utils/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import {
  detailString,
  eventCard,
  eventColumns,
  searchEvent,
  type ConsoleTranslate,
} from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent
import Webhook = Schemas.Webhook

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

export interface PageMessagesProps {
  events: SecurityEvent[]
  webhooks: Webhook[]
  smtpConfigured: boolean
  isLoading: boolean
  isLoadingSmtp: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
  directory: RealmDirectory
}

const EMAIL_TYPES = ['magic_link', 'verify_email', 'reset_password'] as const

const TEMPLATE_ID_DETAIL = 'template_id'

const emailTypeKey = (event: SecurityEvent) => detailString(event, 'email_type')

const emailTypeLabel = (event: SecurityEvent, t: ConsoleTranslate) => {
  const raw = emailTypeKey(event)
  if (!raw) return null
  return EMAIL_TYPES.some((known) => known === raw)
    ? t(`activity.messages.email_types.${raw}`)
    : raw
}

const recipientId = (event: SecurityEvent) =>
  detailString(event, 'user_id') ?? event.actor_id ?? null

export default function PageMessages({
  events,
  webhooks,
  smtpConfigured,
  isLoading,
  isLoadingSmtp,
  isError,
  truncated,
  windowDays,
  windowLimit,
  directory,
}: PageMessagesProps) {
  const { t } = useTranslation(CONSOLE_NAMESPACES)
  const [filter, setFilter] = useState('all')
  const [query, setQuery] = useState('')

  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const filters = useMemo(
    () => [
      { key: 'all', label: t('activity.filter.all') },
      { key: 'delivered', label: t('activity.messages.filters.delivered') },
      { key: 'failed', label: t('activity.messages.filters.failed') },
    ],
    [t]
  )

  const delivered = events.filter((e) => e.event_type === 'email_sent')
  const failed = events.filter((e) => e.event_type === 'email_not_sent')
  const rate = events.length ? Math.round((delivered.length / events.length) * 100) : 0
  const recipients = new Set(events.map((e) => recipientId(e) ?? 'unknown')).size

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: (number | null)[]) =>
    !truncated && series.length > 0 ? series : undefined

  const metrics: Metric[] = [
    {
      key: 'delivered',
      label: t('activity.messages.metrics.delivered'),
      value: t('number', { value: delivered.length }),
      hint: overWindow,
      series: measured(buckets.map((b) => b.filter((e) => e.event_type === 'email_sent').length)),
      tone: 'success',
    },
    {
      key: 'failed',
      label: t('activity.messages.metrics.failed'),
      value: t('number', { value: failed.length }),
      hint: failed.length === 0 ? t('activity.no_failure') : overWindow,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'email_not_sent').length)
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: t('activity.messages.metrics.rate'),
      value: `${rate}%`,
      hint: t('activity.messages.metrics.attempted', { total: events.length }),
      series: measured(
        buckets.map((b) => {
          if (b.length === 0) return null
          const ok = b.filter((e) => e.event_type === 'email_sent').length
          return Math.round((ok / b.length) * 100)
        })
      ),
      tone: 'info',
    },
    {
      key: 'recipients',
      label: t('activity.messages.metrics.recipients'),
      value: t('number', { value: recipients }),
      hint: overWindow,
      series: measured(buckets.map((b) => new Set(b.map((e) => recipientId(e) ?? 'unknown')).size)),
      tone: 'violet',
    },
  ]

  const notices: Notice[] = [
    ...(!isLoadingSmtp && !smtpConfigured
      ? [
          {
            tone: 'warn' as const,
            title: t('activity.messages.notices.no_smtp.title'),
            detail: t('activity.messages.notices.no_smtp.detail'),
          },
        ]
      : []),
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: t('activity.messages.notices.error.title'),
            detail: t('activity.messages.notices.error.detail'),
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
    ...(failed.length > 0
      ? [
          {
            tone: 'error' as const,
            title: t('activity.messages.notices.failed.title', { count: failed.length }),
            detail: t('activity.messages.notices.failed.detail'),
          },
        ]
      : []),
  ]

  const shared = eventColumns(directory, t)

  const recipientColumn: Column<SecurityEvent> = {
    key: 'recipient',
    header: t('activity.messages.columns.recipient'),
    render: (e) => {
      const id = recipientId(e)
      if (!id)
        return (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            {t('activity.event.not_recorded')}
          </span>
        )
      const name = directory.userLabel(id)
      return (
        <div className='min-w-0'>
          <span
            className={
              name
                ? 'text-neutral-700 dark:text-neutral-300'
                : 'font-mono-ui text-xs text-neutral-600 dark:text-neutral-400'
            }
          >
            {name ?? id}
          </span>
          <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {name ? id : t('activity.messages.recipient_account')}
          </p>
        </div>
      )
    },
    sortValue: (e) => directory.userLabel(recipientId(e)) ?? recipientId(e) ?? '',
  }

  const messageColumn: Column<SecurityEvent> = {
    key: 'message',
    header: t('activity.messages.columns.message'),
    render: (e) => (
      <div className='min-w-0'>
        <span>{emailTypeLabel(e, t) ?? t('activity.messages.default_message')}</span>
        <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
          {detailString(e, TEMPLATE_ID_DETAIL) ?? t('activity.messages.no_template')}
        </p>
      </div>
    ),
    sortValue: (e) => emailTypeLabel(e, t) ?? '',
  }

  const filtered = useMemo(() => {
    const byFilter =
      filter === 'delivered'
        ? events.filter((e) => e.event_type === 'email_sent')
        : filter === 'failed'
          ? events.filter((e) => e.event_type === 'email_not_sent')
          : events
    const needle = query.trim().toLowerCase()
    if (!needle) return byFilter
    return byFilter.filter((e) =>
      `${searchEvent(e)} ${emailTypeLabel(e, t) ?? ''}`.toLowerCase().includes(needle)
    )
  }, [events, filter, query, t])

  return (
    <ActivityPage
      title={t('activity.messages.title')}
      description={t('activity.messages.description', { window: windowLabel })}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title={t('activity.messages.journal.title')}
        description={t('activity.messages.journal.description')}
        rows={filtered}
        total={events.length}
        columns={[messageColumn, shared.outcome, recipientColumn, shared.when]}
        card={eventCard(directory, t)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={filter}
        onFilter={setFilter}
        query={query}
        onQuery={setQuery}
        searchPlaceholder={t('activity.messages.journal.search_placeholder')}
        aggregates={{
          message: t('activity.messages.aggregates.attempts', { count: events.length }),
          status: t('activity.messages.aggregates.failed', { total: failed.length }),
          recipient: t('activity.messages.aggregates.recipients', { count: recipients }),
        }}
        emptyLabel={t('activity.messages.journal.empty_label')}
        emptyHint={t('activity.messages.journal.empty_hint', { window: windowLabel })}
      />

      <Section
        title={t('activity.messages.webhooks.title')}
        description={t('activity.messages.webhooks.description')}
        contained={false}
        action={
          <span className='tnum text-[11px] text-neutral-400 dark:text-neutral-500'>
            {t('activity.messages.webhooks.count', { count: webhooks.length })}
          </span>
        }
      >
        <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
          {webhooks.length === 0 ? (
            <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
              {t('activity.messages.webhooks.empty')}
            </div>
          ) : (
            webhooks.map((webhook) => (
              <div key={webhook.id} className='flex items-center gap-3 px-3 py-2 text-[13px]'>
                <div className='min-w-0 flex-1'>
                  <span className='block truncate'>{webhook.name || webhook.endpoint}</span>
                  <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
                    {webhook.endpoint}
                  </p>
                </div>
                <Pill tone='neutral' mono>
                  {t('activity.messages.webhooks.triggers', {
                    count: webhook.subscribers.length,
                  })}
                </Pill>
                <span
                  className='tnum w-28 shrink-0 text-right text-[11px] text-neutral-400 dark:text-neutral-500'
                  title={webhook.triggered_at ? formatTimestamp(webhook.triggered_at) : undefined}
                >
                  {webhook.triggered_at
                    ? formatRelative(webhook.triggered_at)
                    : t('activity.messages.webhooks.never_fired')}
                </span>
              </div>
            ))
          )}
        </div>
      </Section>
    </ActivityPage>
  )
}
