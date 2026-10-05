import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
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
  type ConsoleTranslate,
} from './event-journal'
import { securityEventColumnFilters } from '@/pages/iam/seawatch/event-filter-fields'
import type { EventJournal } from '../feature/use-event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent
import Webhook = Schemas.Webhook

const CONSOLE_NAMESPACES = ['console', 'seawatch'] as const

export interface MessageCounts {
  delivered: number
  failed: number
  total: number
}

export interface PageMessagesProps {
  events: SecurityEvent[]
  counts: MessageCounts
  journal: EventJournal
  webhooks: Webhook[]
  webhookTotal: number
  webhooksHref: string
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
  counts,
  journal,
  webhooks,
  webhookTotal,
  webhooksHref,
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

  const windowLabel = t('activity.window', { count: windowDays })
  const overWindow = t('activity.over_window', { window: windowLabel })

  const tabs = [
    { value: '', label: t('activity.filter.all') },
    { value: 'email_sent', label: t('activity.messages.filters.delivered') },
    { value: 'email_not_sent', label: t('activity.messages.filters.failed') },
  ]

  const rate = counts.total ? Math.round((counts.delivered / counts.total) * 100) : 0
  const recipients = new Set(events.map((e) => recipientId(e) ?? 'unknown')).size

  const buckets = useMemo(() => bucketPerDay(events, windowDays), [events, windowDays])

  const measured = (series: (number | null)[]) =>
    !truncated && series.length > 0 ? series : undefined
  const counted = (series: number[], value: number) =>
    !truncated && series.length > 0 ? series : [value, value]

  const metrics: Metric[] = [
    {
      key: 'delivered',
      label: t('activity.messages.metrics.delivered'),
      value: t('number', { value: counts.delivered }),
      hint: overWindow,
      series: counted(
        buckets.map((b) => b.filter((e) => e.event_type === 'email_sent').length),
        counts.delivered
      ),
      tone: 'success',
    },
    {
      key: 'failed',
      label: t('activity.messages.metrics.failed'),
      value: t('number', { value: counts.failed }),
      hint: counts.failed === 0 ? t('activity.no_failure') : overWindow,
      series: counted(
        buckets.map((b) => b.filter((e) => e.event_type === 'email_not_sent').length),
        counts.failed
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: t('activity.messages.metrics.rate'),
      value: `${rate}%`,
      hint: t('activity.messages.metrics.attempted', { total: counts.total }),
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
    ...(counts.failed > 0
      ? [
          {
            tone: 'error' as const,
            title: t('activity.messages.notices.failed.title', { count: counts.failed }),
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
  }

  const journalColumns = [
    messageColumn,
    { ...shared.outcome, sortKey: 'status' },
    {
      ...recipientColumn,
      filters: securityEventColumnFilters().actor.map((field) => ({
        ...field,
        label: t('activity.messages.columns.recipient'),
      })),
    },
    { ...shared.when, sortKey: 'timestamp' },
  ]

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
        rows={journal.events}
        pagination={journal.pagination}
        listing={journal.listing}
        tabs={tabs}
        search={{ placeholder: t('seawatch:stream.search_placeholder') }}
        columns={journalColumns}
        card={eventCard(directory, t)}
        getKey={(e) => e.id}
        loading={isLoading || journal.isLoading}
        aggregates={{
          message: t('activity.messages.aggregates.attempts', {
            count: journal.firstCount + journal.secondCount,
          }),
          status: t('activity.messages.aggregates.failed', { total: journal.secondCount }),
        }}
        emptyLabel={t('activity.messages.journal.empty_label')}
        emptyHint={t('activity.messages.journal.empty_hint')}
      />

      <Section
        title={t('activity.messages.webhooks.title')}
        description={t('activity.messages.webhooks.description')}
        contained={false}
        action={
          webhookTotal > webhooks.length ? (
            <span className='flex items-center gap-2 text-[11px] text-neutral-400 dark:text-neutral-500'>
              <span className='tnum'>
                {t('activity.messages.webhooks.showing', {
                  shown: webhooks.length,
                  count: webhookTotal,
                })}
              </span>
              <Link
                to={webhooksHref}
                className='underline-offset-2 hover:text-fk-primary-text hover:underline'
              >
                {t('activity.messages.webhooks.view_all')}
              </Link>
            </span>
          ) : (
            <span className='tnum text-[11px] text-neutral-400 dark:text-neutral-500'>
              {t('activity.messages.webhooks.count', { count: webhookTotal })}
            </span>
          )
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
