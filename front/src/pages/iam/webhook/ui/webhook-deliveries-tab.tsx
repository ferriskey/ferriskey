import { useState } from 'react'
import { toast } from 'sonner'
import { RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button, Column, DataView, Pill, Section, Segmented, type ViewMode } from '@/components/kit'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  useGetWebhookDeliveries,
  useGetWebhookDelivery,
  useRetryWebhookDelivery,
} from '@/api/webhook.api'
import { Schemas } from '@/api/api.client'
import { formatRelative, formatTimestamp } from '@/utils/format-date'
import { apiErrorMessage } from '@/lib/api-error'
import {
  DELIVERY_STATUS_FILTERS,
  describeDeliveryStatus,
  isReplayable,
} from '../webhook-delivery-status'

import DeliverySummary = Schemas.DeliverySummary

const PAGE_SIZE = 25
const ALL_STATUS = 'all'
const DELIVERY_VIEW: ViewMode = 'list'

export interface WebhookDeliveriesTabProps {
  realm: string
  webhookId: string
}

function outcomeOf(delivery: DeliverySummary): string {
  if (delivery.last_status_code) return `HTTP ${delivery.last_status_code}`
  if (delivery.last_error_code) return delivery.last_error_code
  return '—'
}

export default function WebhookDeliveriesTab({ realm, webhookId }: WebhookDeliveriesTabProps) {
  const { t } = useTranslation('webhook')
  const [status, setStatus] = useState(ALL_STATUS)
  const [offset, setOffset] = useState(0)
  const [openDeliveryId, setOpenDeliveryId] = useState<string | null>(null)

  const { data, isLoading, isError } = useGetWebhookDeliveries({
    realm,
    webhookId,
    status: status === ALL_STATUS ? undefined : status,
    limit: PAGE_SIZE,
    offset,
  })

  const { data: detail } = useGetWebhookDelivery({
    realm,
    webhookId,
    deliveryId: openDeliveryId,
  })

  const { mutate: retryDelivery, isPending: isRetrying } = useRetryWebhookDelivery()

  const deliveries = data?.data ?? []
  const total = data?.total ?? 0
  const from = total === 0 ? 0 : offset + 1
  const to = offset + deliveries.length

  const statusOf = (value: string) => {
    const descriptor = describeDeliveryStatus(value)
    return { tone: descriptor.tone, label: descriptor.labelKey ? t(descriptor.labelKey) : value }
  }

  const filters = DELIVERY_STATUS_FILTERS.map((filter) => ({
    key: filter.key,
    label: t(filter.labelKey),
  }))

  const onFilterChange = (next: string) => {
    setStatus(next)
    setOffset(0)
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
      sortValue: (delivery) => delivery.status,
    },
    {
      key: 'event',
      header: t('delivery.columns.event'),
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-700 dark:text-neutral-300'>
          {delivery.event}
        </span>
      ),
      sortValue: (delivery) => delivery.event,
    },
    {
      key: 'attempts',
      header: t('delivery.columns.attempts'),
      render: (delivery) => <span className='tnum'>{delivery.attempt_count}</span>,
      sortValue: (delivery) => delivery.attempt_count,
    },
    {
      key: 'outcome',
      header: t('delivery.columns.outcome'),
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-500 dark:text-neutral-400'>
          {outcomeOf(delivery)}
        </span>
      ),
      sortValue: (delivery) => outcomeOf(delivery),
    },
    {
      key: 'created_at',
      header: t('delivery.columns.created_at'),
      align: 'right',
      render: (delivery) => (
        <div className='min-w-0 whitespace-nowrap'>
          <span className='text-neutral-700 dark:text-neutral-300'>
            {formatRelative(delivery.created_at)}
          </span>
          <p className='tnum truncate font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
            {formatTimestamp(delivery.created_at)}
          </p>
        </div>
      ),
      sortValue: (delivery) => delivery.created_at,
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

  const failed = deliveries.filter((delivery) => delivery.status === 'failed').length

  return (
    <>
      <Section
        title={t('delivery.title')}
        description={t('delivery.description')}
        contained={false}
        action={<Segmented items={filters} value={status} onChange={onFilterChange} />}
      >
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
            aggregates={
              failed > 0
                ? { status: t('delivery.failed_on_page', { total: failed }) }
                : undefined
            }
            emptyLabel={
              status === ALL_STATUS ? t('delivery.empty.all.label') : t('delivery.empty.filtered.label')
            }
            emptyHint={
              status === ALL_STATUS ? t('delivery.empty.all.hint') : t('delivery.empty.filtered.hint')
            }
            emptyAction={
              status === ALL_STATUS ? undefined : (
                <Button variant='outline' onClick={() => onFilterChange(ALL_STATUS)}>
                  {t('delivery.show_all')}
                </Button>
              )
            }
          />
        )}

        {total > PAGE_SIZE && (
          <div className='mt-3 flex items-center justify-between px-1'>
            <span className='text-sm text-muted-foreground'>
              {t('delivery.range', { from, to, total })}
            </span>
            <div className='flex gap-2'>
              <Button
                variant='outline'
                size='sm'
                disabled={offset === 0}
                onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              >
                {t('delivery.previous')}
              </Button>
              <Button
                variant='outline'
                size='sm'
                disabled={to >= total}
                onClick={() => setOffset(offset + PAGE_SIZE)}
              >
                {t('delivery.next')}
              </Button>
            </div>
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
