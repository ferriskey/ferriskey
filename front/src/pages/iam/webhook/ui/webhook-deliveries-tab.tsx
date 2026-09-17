import { useState } from 'react'
import { toast } from 'sonner'
import { RotateCcw } from 'lucide-react'
import { Button, Column, DataView, Pill, Section, Segmented } from '@/components/kit'
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
  const [status, setStatus] = useState('all')
  const [offset, setOffset] = useState(0)
  const [openDeliveryId, setOpenDeliveryId] = useState<string | null>(null)

  const { data, isLoading, isError } = useGetWebhookDeliveries({
    realm,
    webhookId,
    status: status === 'all' ? undefined : status,
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
        onSuccess: () => toast.success('Delivery queued for another attempt'),
        onError: (error: unknown) => {
          if ((error as { status?: number })?.status === 409) {
            toast.error('This delivery is still in flight. Wait for it to finish.')
            return
          }
          toast.error(apiErrorMessage(error, 'Could not queue this delivery'))
        },
      }
    )
  }

  const columns: Column<DeliverySummary>[] = [
    {
      key: 'status',
      header: 'Status',
      render: (delivery) => {
        const descriptor = describeDeliveryStatus(delivery.status)
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
      header: 'Event',
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-700 dark:text-neutral-300'>
          {delivery.event}
        </span>
      ),
      sortValue: (delivery) => delivery.event,
    },
    {
      key: 'attempts',
      header: 'Attempts',
      render: (delivery) => <span className='tnum'>{delivery.attempt_count}</span>,
      sortValue: (delivery) => delivery.attempt_count,
    },
    {
      key: 'outcome',
      header: 'Outcome',
      render: (delivery) => (
        <span className='font-mono-ui text-[12px] text-neutral-500 dark:text-neutral-400'>
          {outcomeOf(delivery)}
        </span>
      ),
      sortValue: (delivery) => outcomeOf(delivery),
    },
    {
      key: 'created_at',
      header: 'When',
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
            Inspect
          </Button>
          {isReplayable(delivery.status) && (
            <Button
              variant='outline'
              size='sm'
              disabled={isRetrying}
              onClick={() => onRetry(delivery)}
            >
              <RotateCcw className='mr-1 size-3.5' />
              Send again
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
        title='Deliveries'
        description='Every attempt FerrisKey made to call this endpoint, newest first.'
        contained={false}
        action={
          <Segmented
            items={DELIVERY_STATUS_FILTERS}
            value={status}
            onChange={onFilterChange}
          />
        }
      >
        {isError ? (
          <div className='rounded-md border border-fk-danger-border bg-fk-danger-soft/40 px-3 py-2.5 text-sm text-fk-danger'>
            Delivery history is unavailable. Try again in a moment.
          </div>
        ) : (
          <DataView
            rows={deliveries}
            columns={columns}
            card={{
              title: (delivery) => delivery.event,
              subtitle: (delivery) => formatTimestamp(delivery.created_at),
              badges: (delivery) => {
                const descriptor = describeDeliveryStatus(delivery.status)
                return (
                  <Pill tone={descriptor.tone} mono>
                    {descriptor.label}
                  </Pill>
                )
              },
              footer: (delivery) => outcomeOf(delivery),
            }}
            getKey={(delivery) => delivery.id}
            view='list'
            loading={isLoading}
            aggregates={
              failed > 0 ? { status: `${failed} failed on this page` } : undefined
            }
            emptyLabel={status === 'all' ? 'No deliveries yet' : 'No delivery matches this filter'}
            emptyHint={
              status === 'all'
                ? 'Deliveries appear here as soon as this webhook fires.'
                : 'Change the filter to see the other deliveries.'
            }
            emptyAction={
              status === 'all' ? undefined : (
                <Button variant='outline' onClick={() => onFilterChange('all')}>
                  Show all
                </Button>
              )
            }
          />
        )}

        {total > PAGE_SIZE && (
          <div className='mt-3 flex items-center justify-between px-1'>
            <span className='text-sm text-muted-foreground'>
              {from}-{to} of {total}
            </span>
            <div className='flex gap-2'>
              <Button
                variant='outline'
                size='sm'
                disabled={offset === 0}
                onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              >
                Previous
              </Button>
              <Button
                variant='outline'
                size='sm'
                disabled={to >= total}
                onClick={() => setOffset(offset + PAGE_SIZE)}
              >
                Next
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
            <DialogTitle>{detail?.data.event ?? 'Delivery'}</DialogTitle>
            <DialogDescription>
              {detail ? formatTimestamp(detail.data.created_at) : 'Loading the delivery…'}
            </DialogDescription>
          </DialogHeader>

          {detail && (
            <div className='space-y-4'>
              <div className='flex flex-wrap items-center gap-2'>
                <Pill tone={describeDeliveryStatus(detail.data.status).tone} mono>
                  {describeDeliveryStatus(detail.data.status).label}
                </Pill>
                <span className='font-mono-ui text-[12px] text-neutral-500 dark:text-neutral-400'>
                  {detail.data.attempt_count} attempt
                  {detail.data.attempt_count === 1 ? '' : 's'} · {outcomeOf(detail.data)}
                </span>
              </div>

              {detail.data.last_error_detail && (
                <div className='rounded-md border border-fk-danger-border bg-fk-danger-soft/40 px-3 py-2.5 text-sm text-fk-danger'>
                  {detail.data.last_error_detail}
                </div>
              )}

              <div>
                <p className='mb-1.5 text-xs text-neutral-500 dark:text-neutral-400'>Payload sent</p>
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
