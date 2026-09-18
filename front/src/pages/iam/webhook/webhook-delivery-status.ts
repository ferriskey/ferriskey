import type { PillTone } from '@/components/kit'

interface DeliveryStatusDescriptor {
  labelKey: string | null
  tone: PillTone
  terminal: boolean
}

const STATUSES: Record<string, DeliveryStatusDescriptor> = {
  pending: { labelKey: 'delivery.status.pending', tone: 'amber', terminal: false },
  delivering: { labelKey: 'delivery.status.delivering', tone: 'info', terminal: false },
  succeeded: { labelKey: 'delivery.status.succeeded', tone: 'success', terminal: true },
  failed: { labelKey: 'delivery.status.failed', tone: 'danger', terminal: true },
}

export const DELIVERY_STATUS_FILTERS = [
  { key: 'all', labelKey: 'delivery.filters.all' },
  { key: 'failed', labelKey: 'delivery.status.failed' },
  { key: 'pending', labelKey: 'delivery.status.pending' },
  { key: 'succeeded', labelKey: 'delivery.status.succeeded' },
]

export function describeDeliveryStatus(status: string): DeliveryStatusDescriptor {
  return STATUSES[status] ?? { labelKey: null, tone: 'neutral', terminal: false }
}

export function isReplayable(status: string): boolean {
  return describeDeliveryStatus(status).terminal
}
