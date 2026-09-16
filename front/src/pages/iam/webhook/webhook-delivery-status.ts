import type { PillTone } from '@/components/kit'

interface DeliveryStatusDescriptor {
  label: string
  tone: PillTone
  terminal: boolean
}

const STATUSES: Record<string, DeliveryStatusDescriptor> = {
  pending: { label: 'Queued', tone: 'amber', terminal: false },
  delivering: { label: 'Sending', tone: 'info', terminal: false },
  succeeded: { label: 'Delivered', tone: 'success', terminal: true },
  failed: { label: 'Failed', tone: 'danger', terminal: true },
}

export const DELIVERY_STATUS_FILTERS = [
  { key: 'all', label: 'All' },
  { key: 'failed', label: 'Failed' },
  { key: 'pending', label: 'Queued' },
  { key: 'succeeded', label: 'Delivered' },
]

export function describeDeliveryStatus(status: string): DeliveryStatusDescriptor {
  return STATUSES[status] ?? { label: status, tone: 'neutral', terminal: false }
}

export function isReplayable(status: string): boolean {
  return describeDeliveryStatus(status).terminal
}
