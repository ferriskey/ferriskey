import { useMemo } from 'react'
import { Schemas } from '@/api/api.client'
import {
  useGetSecurityEvents,
  useSecurityEventCount,
  type SecurityEventsFilter,
} from '@/api/sea-watch.api'

import EventStatus = Schemas.EventStatus
import SecurityEventType = Schemas.SecurityEventType

export const WINDOW_DAYS = 7
export const WINDOW_LIMIT = 100

export interface WindowRange {
  from_timestamp: string
  to_timestamp: string
}

export interface WindowEvents {
  events: Schemas.SecurityEvent[]
  total: number
  range: WindowRange
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
}

export function useWindowRange(): WindowRange {
  return useMemo(() => {
    const to = new Date()
    const from = new Date(to.getTime() - WINDOW_DAYS * 24 * 60 * 60 * 1000)
    return { from_timestamp: from.toISOString(), to_timestamp: to.toISOString() }
  }, [])
}

export function useWindowEvents(
  realm: string,
  eventTypes?: readonly SecurityEventType[],
  status?: EventStatus
): WindowEvents {
  const range = useWindowRange()

  const { data, isLoading, isError } = useGetSecurityEvents({
    realm,
    query: {
      ...range,
      limit: WINDOW_LIMIT,
      order_by: 'timestamp',
      event_types: eventTypes ? eventTypes.join(',') : undefined,
      status,
    },
  })

  const events = useMemo(() => data?.data ?? [], [data])
  const total = data?.metadata.total ?? 0

  return {
    events,
    total,
    range,
    isLoading,
    isError,
    truncated: total > events.length,
    windowDays: WINDOW_DAYS,
    windowLimit: WINDOW_LIMIT,
  }
}

export function useWindowCount(realm: string, range: WindowRange, filter: SecurityEventsFilter) {
  return useSecurityEventCount({ realm, filter: { ...range, ...filter } }).count
}

const dayKey = (value: string) => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
}

export const bucketPerDay = (events: Schemas.SecurityEvent[], days: number) => {
  const today = new Date()
  return Array.from({ length: days }, (_, index) => {
    const day = new Date(today)
    day.setDate(today.getDate() - (days - 1 - index))
    const key = dayKey(day.toISOString())
    return events.filter((event) => dayKey(event.timestamp) === key)
  })
}
