import { useMemo, useState } from 'react'
import { MetricsBand, Pill, Section } from '@/components/kit'
import type { Column, Metric } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import type { RealmDirectory } from '@/next/shared/use-realm-directory'
import { formatRelative, formatTimestamp } from '@/next/shared/format-date'
import { bucketPerDay } from '../feature/use-window-events'
import { ActivityPage, NoticeList, type Notice } from './activity-notices'
import { detailString, eventCard, eventColumns, searchEvent } from './event-journal'
import { JournalSection } from './journal-section'

import SecurityEvent = Schemas.SecurityEvent
import Webhook = Schemas.Webhook

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

const filters = [
  { key: 'all', label: 'All' },
  { key: 'delivered', label: 'Delivered' },
  { key: 'failed', label: 'Failed' },
]

const emailTypeLabels: Record<string, string> = {
  magic_link: 'Magic link',
  verify_email: 'Email verification',
  reset_password: 'Password reset',
}

const emailType = (event: SecurityEvent) => {
  const raw = detailString(event, 'email_type')
  if (!raw) return null
  return emailTypeLabels[raw] ?? raw
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
  const [filter, setFilter] = useState('all')
  const [query, setQuery] = useState('')

  const windowLabel = `last ${windowDays} days`

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
      label: 'Emails delivered',
      value: delivered.length.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(buckets.map((b) => b.filter((e) => e.event_type === 'email_sent').length)),
      tone: 'success',
    },
    {
      key: 'failed',
      label: 'Deliveries failed',
      value: failed.length.toLocaleString(),
      hint: failed.length === 0 ? 'no failure recorded' : `over the ${windowLabel}`,
      series: measured(
        buckets.map((b) => b.filter((e) => e.event_type === 'email_not_sent').length)
      ),
      tone: 'brand',
    },
    {
      key: 'rate',
      label: 'Delivery rate',
      value: `${rate}%`,
      hint: `${events.length.toLocaleString()} attempted`,
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
      label: 'Distinct recipients',
      value: recipients.toLocaleString(),
      hint: `over the ${windowLabel}`,
      series: measured(buckets.map((b) => new Set(b.map((e) => recipientId(e) ?? 'unknown')).size)),
      tone: 'violet',
    },
  ]

  const notices: Notice[] = [
    ...(!isLoadingSmtp && !smtpConfigured
      ? [
          {
            tone: 'warn' as const,
            title: 'No SMTP configuration could be read for this realm',
            detail:
              'Without a mail server, no transactional email can leave the realm — which is what an empty journal below would mean.',
          },
        ]
      : []),
    ...(isError
      ? [
          {
            tone: 'error' as const,
            title: 'Delivery journal unavailable',
            detail: 'We could not fetch the delivery events of this realm. Please try again later.',
          },
        ]
      : []),
    ...(truncated
      ? [
          {
            tone: 'warn' as const,
            title: `Capped at ${windowLimit} events`,
            detail: `The realm recorded more over the ${windowLabel}; the figures above cover the ${windowLimit} most recent only.`,
          },
        ]
      : []),
    ...(failed.length > 0
      ? [
          {
            tone: 'error' as const,
            title: `${failed.length} email${failed.length > 1 ? 's' : ''} could not be delivered`,
            detail: 'Each failed row carries the reason the mail server returned.',
          },
        ]
      : []),
  ]

  const shared = eventColumns(directory)

  const recipientColumn: Column<SecurityEvent> = {
    key: 'recipient',
    header: 'Recipient',
    render: (e) => {
      const id = recipientId(e)
      if (!id)
        return (
          <span className='text-xs text-neutral-400 dark:text-neutral-500'>
            not recorded
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
            {name ? id : 'account'}
          </p>
        </div>
      )
    },
    sortValue: (e) => directory.userLabel(recipientId(e)) ?? recipientId(e) ?? '',
  }

  const messageColumn: Column<SecurityEvent> = {
    key: 'message',
    header: 'Message',
    render: (e) => (
      <div className='min-w-0'>
        <span>{emailType(e) ?? 'Transactional email'}</span>
        <p className='truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
          {detailString(e, 'template_id') ?? 'no template recorded'}
        </p>
      </div>
    ),
    sortValue: (e) => emailType(e) ?? '',
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
      `${searchEvent(e)} ${emailType(e) ?? ''}`.toLowerCase().includes(needle)
    )
  }, [events, filter, query])

  return (
    <ActivityPage
      title='Message delivery'
      description={`Transactional emails this realm attempted to send over the ${windowLabel}, and the webhook endpoints it notifies.`}
    >
      <NoticeList notices={notices} />

      <MetricsBand metrics={metrics} />

      <JournalSection
        title='Email deliveries'
        description='One line per email the realm handed to its mail server, delivered or not.'
        rows={filtered}
        total={events.length}
        columns={[messageColumn, shared.outcome, recipientColumn, shared.when]}
        card={eventCard(directory)}
        getKey={(e) => e.id}
        loading={isLoading}
        filters={filters}
        filter={filter}
        onFilter={setFilter}
        query={query}
        onQuery={setQuery}
        searchPlaceholder='Search by message, account…'
        aggregates={{
          message: `${events.length} attempt${events.length !== 1 ? 's' : ''}`,
          status: `${failed.length} failed`,
          recipient: `${recipients} recipient${recipients !== 1 ? 's' : ''}`,
        }}
        emptyLabel='No email delivery'
        emptyHint={`This journal fills as the realm sends magic links, verification emails and password resets. Nothing was attempted over the ${windowLabel}.`}
      />

      <Section
        title='Webhook endpoints'
        description='The endpoints this realm notifies, and the last time each one fired. Individual deliveries are not journalled, so no per-delivery status exists to show.'
        contained={false}
        action={
          <span className='tnum text-[11px] text-neutral-400 dark:text-neutral-500'>
            {webhooks.length} endpoint{webhooks.length !== 1 ? 's' : ''}
          </span>
        }
      >
        <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
          {webhooks.length === 0 ? (
            <div className='px-3 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400'>
              No webhook endpoint is registered for this realm.
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
                  {webhook.subscribers.length} trigger
                  {webhook.subscribers.length !== 1 ? 's' : ''}
                </Pill>
                <span
                  className='tnum w-28 shrink-0 text-right text-[11px] text-neutral-400 dark:text-neutral-500'
                  title={webhook.triggered_at ? formatTimestamp(webhook.triggered_at) : undefined}
                >
                  {webhook.triggered_at
                    ? formatRelative(webhook.triggered_at)
                    : 'never fired'}
                </span>
              </div>
            ))
          )}
        </div>
      </Section>
    </ActivityPage>
  )
}
