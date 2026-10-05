import { useMemo, useState } from 'react'
import { toast } from 'sonner'
import { RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Column,
  DataView,
  ListingToolbar,
  PaginationBar,
  Pill,
  Section,
  usePagedListing,
  type ViewMode,
} from '@/components/kit'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  DELIVERY_FILTER_KEYS,
  useGetWebhookDeliveries,
  useGetWebhookDelivery,
  useRetryWebhookDelivery,
  useWebhookDeliveryCount,
  type WebhookDeliveriesQuery,
} from '@/api/webhook.api'
import { Schemas } from '@/api/api.client'
import { catalogedTriggers } from '@/constants/webhook-utils'
import { formatRelative, formatTimestamp } from '@/utils/format-date'
import { apiErrorMessage } from '@/lib/api-error'
import { DELIVERY_STATUSES, describeDeliveryStatus, isReplayable } from '../webhook-delivery-status'

import DeliverySummary = Schemas.DeliverySummary

const DELIVERY_VIEW: ViewMode = 'list'
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
const EVENT_OPTIONS = (Object.keys(catalogedTriggers) as Schemas.WebhookTrigger[])
  .sort()
  .map((event) => ({ value: event, label: event }))

export interface WebhookDeliveriesTabProps {
  realm: string
  webhookId: string
}

function outcomeOf(delivery: DeliverySummary): string {
  if (delivery.last_status_code) return `HTTP ${delivery.last_status_code}`
  if (delivery.last_error_code) return delivery.last_error_code
  return '—'
}

function deliveriesQuery(apiQuery: Record<string, string | number>): WebhookDeliveriesQuery {
  const { resource_id: resourceId, ...rest } = apiQuery
  const query = rest as WebhookDeliveriesQuery
  if (typeof resourceId === 'string' && UUID_PATTERN.test(resourceId.trim())) {
    return { ...query, resource_id: resourceId.trim() }
  }
  return query
}

function TimestampCell({ value }: { value?: string | null }) {
  if (!value) {
    return <span className='text-neutral-400 dark:text-neutral-500'>—</span>
  }
  return (
    <div className='min-w-0 whitespace-nowrap'>
      <span className='text-neutral-700 dark:text-neutral-300'>{formatRelative(value)}</span>
      <p className='tnum truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
        {formatTimestamp(value)}
      </p>
    </div>
  )
}

export default function WebhookDeliveriesTab({ realm, webhookId }: WebhookDeliveriesTabProps) {
  const { t } = useTranslation('webhook')
  const listing = usePagedListing(DELIVERY_FILTER_KEYS)
  const [openDeliveryId, setOpenDeliveryId] = useState<string | null>(null)

  const query = useMemo(() => deliveriesQuery(listing.apiQuery), [listing.apiQuery])
  const { data, isLoading, isError } = useGetWebhookDeliveries({
    realm,
    webhookId,
    query,
    keepPrevious: true,
  })
  const { count: failedTotal } = useWebhookDeliveryCount({
    realm,
    webhookId,
    filter: { status: 'failed' },
  })

  const { data: detail } = useGetWebhookDelivery({
    realm,
    webhookId,
    deliveryId: openDeliveryId,
  })

  const { mutate: retryDelivery, isPending: isRetrying } = useRetryWebhookDelivery()

  const deliveries = data?.data ?? []
  const pagination = data?.metadata
  const narrowed = Object.values(listing.state.filters).some(Boolean)

  const statusOf = (value: string) => {
    const descriptor = describeDeliveryStatus(value)
    return { tone: descriptor.tone, label: descriptor.labelKey ? t(descriptor.labelKey) : value }
  }

  const onRetry = (delivery: DeliverySummary) => {
    retryDelivery(
      {
        path: { realm_name: realm, webhook_id: webhookId, delivery_id: delivery.id },
      },
      {
        onSuccess: () => toast.success(t('delivery.toast.retry_queued')),
        onError: (error: unknown) => {
          if ((error as { status?: number })?.status === 409) {
            toast.error(t('delivery.toast.in_flight'))
            return
          }
          toast.error(apiErrorMessage(error, t('delivery.retry_failed')))
        },
      }
    )
  }

  const columns: Column<DeliverySummary>[] = [
    {
      key: 'status',
      header: t('delivery.columns.status'),
      render: (delivery) => {
        const descriptor = statusOf(delivery.status)
        return (
          <Pill tone={descriptor.tone} mono>
            {descriptor.label}
          </Pill>
        )
      },
      sortKey: 'status',
      filters: [
        {
          kind: 'enum',
          key: 'status',
          label: t('delivery.filter_fields.status'),
          options: DELIVERY_STATUSES.map((status) => ({
            value: status,
            label: statusOf(status).label,
          })),
        },
      ],
    },
    {
      key: 'event',
      header: t('delivery.columns.event'),
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-700 dark:text-neutral-300'>
          {delivery.event}
        </span>
      ),
      filters: [
        {
          kind: 'enum',
          key: 'event',
          label: t('delivery.filter_fields.event'),
          options: EVENT_OPTIONS,
        },
        {
          kind: 'text',
          key: 'resource_id',
          label: t('delivery.filter_fields.resource_id'),
        },
      ],
    },
    {
      key: 'attempts',
      header: t('delivery.columns.attempts'),
      render: (delivery) => <span className='tnum'>{delivery.attempt_count}</span>,
      sortKey: 'attempt_count',
    },
    {
      key: 'outcome',
      header: t('delivery.columns.outcome'),
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-500 dark:text-neutral-400'>
          {outcomeOf(delivery)}
        </span>
      ),
    },
    {
      key: 'last_attempt_at',
      header: t('delivery.columns.last_attempt_at'),
      align: 'right',
      render: (delivery) => <TimestampCell value={delivery.last_attempt_at} />,
      sortKey: 'last_attempt_at',
    },
    {
      key: 'updated_at',
      header: t('delivery.columns.updated_at'),
      align: 'right',
      render: (delivery) => <TimestampCell value={delivery.updated_at} />,
      sortKey: 'updated_at',
    },
    {
      key: 'created_at',
      header: t('delivery.columns.created_at'),
      align: 'right',
      render: (delivery) => <TimestampCell value={delivery.created_at} />,
      sortKey: 'created_at',
      filters: [
        {
          kind: 'date-range',
          fromKey: 'created_from',
          toKey: 'created_to',
          label: t('delivery.filter_fields.created'),
        },
      ],
    },
    {
      key: 'actions',
      header: '',
      align: 'right',
      render: (delivery) => (
        <div className='flex justify-end gap-2'>
          <Button variant='outline' size='sm' onClick={() => setOpenDeliveryId(delivery.id)}>
            {t('delivery.inspect')}
          </Button>
          {isReplayable(delivery.status) && (
            <Button
              variant='outline'
              size='sm'
              disabled={isRetrying}
              onClick={() => onRetry(delivery)}
            >
              <RotateCcw className='mr-1 size-3.5' />
              {t('delivery.retry')}
            </Button>
          )}
        </div>
      ),
    },
  ]

  return (
    <>
      <Section title={t('delivery.title')} description={t('delivery.description')} contained={false}>
        <ListingToolbar listing={listing} columns={columns} className='mb-3' />
        {isError ? (
          <div className='rounded-md border border-fk-danger-border bg-fk-danger-soft/40 px-3 py-2.5 text-sm text-fk-danger'>
            {t('delivery.error')}
          </div>
        ) : (
          <DataView
            rows={deliveries}
            columns={columns}
            card={{
              title: (delivery) => delivery.event,
              subtitle: (delivery) => formatTimestamp(delivery.created_at),
              badges: (delivery) => {
                const descriptor = statusOf(delivery.status)
                return (
                  <Pill tone={descriptor.tone} mono>
                    {descriptor.label}
                  </Pill>
                )
              },
              footer: (delivery) => outcomeOf(delivery),
            }}
            getKey={(delivery) => delivery.id}
            view={DELIVERY_VIEW}
            loading={isLoading}
            listing={listing}
            sort={listing.state.sort}
            onSortChange={listing.setSort}
            aggregates={
              failedTotal > 0
                ? { status: t('delivery.failed_total', { count: failedTotal }) }
                : undefined
            }
            emptyLabel={
              narrowed ? t('delivery.empty.filtered.label') : t('delivery.empty.all.label')
            }
            emptyHint={narrowed ? t('delivery.empty.filtered.hint') : t('delivery.empty.all.hint')}
            emptyAction={
              narrowed ? (
                <Button variant='outline' onClick={listing.clearFilters}>
                  {t('delivery.show_all')}
                </Button>
              ) : undefined
            }
          />
        )}

        {pagination && !isError && (
          <div className='mt-3 px-1'>
            <PaginationBar pagination={pagination} onPageChange={listing.setPage} />
          </div>
        )}
      </Section>

      <Dialog
        open={Boolean(openDeliveryId)}
        onOpenChange={(open) => !open && setOpenDeliveryId(null)}
      >
        <DialogContent className='max-w-2xl'>
          <DialogHeader>
            <DialogTitle>{detail?.data.event ?? t('delivery.dialog.title')}</DialogTitle>
            <DialogDescription>
              {detail
                ? formatTimestamp(detail.data.created_at)
                : t('delivery.dialog.loading')}
            </DialogDescription>
          </DialogHeader>

          {detail && (
            <div className='space-y-4'>
              <div className='flex flex-wrap items-center gap-2'>
                <Pill tone={statusOf(detail.data.status).tone} mono>
                  {statusOf(detail.data.status).label}
                </Pill>
                <span className='font-mono-ui text-[12px] text-neutral-500 dark:text-neutral-400'>
                  {t('delivery.dialog.attempts', { count: detail.data.attempt_count })} ·{' '}
                  {outcomeOf(detail.data)}
                </span>
              </div>

              {detail.data.last_error_detail && (
                <div className='rounded-md border border-fk-danger-border bg-fk-danger-soft/40 px-3 py-2.5 text-sm text-fk-danger'>
                  {detail.data.last_error_detail}
                </div>
              )}

              <div>
                <p className='mb-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
                  {t('delivery.dialog.payload')}
                </p>
                <pre className='max-h-80 overflow-auto rounded-md bg-neutral-50 p-3 font-mono-ui text-[12px] leading-relaxed dark:bg-fk-raised'>
                  {JSON.stringify(detail.data.payload, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
