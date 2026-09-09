import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Schemas } from '@/api/api.client'

import SecurityEventType = Schemas.SecurityEventType

export const WINDOW_DAYS = 7
export const WINDOW_LIMIT = 500

export interface WindowEvents {
  events: Schemas.SecurityEvent[]
  isLoading: boolean
  isError: boolean
  truncated: boolean
  windowDays: number
  windowLimit: number
}

export function useWindowEvents(
  realm: string,
  eventTypes?: readonly SecurityEventType[]
): WindowEvents {
  const key = eventTypes ? eventTypes.join(',') : undefined

  const range = useMemo(() => {
    const to = new Date()
    const from = new Date(to.getTime() - WINDOW_DAYS * 24 * 60 * 60 * 1000)
    return { from: from.toISOString(), to: to.toISOString() }
  }, [])

  const { data, isLoading, isError } = useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/seawatch/v1/security-events', {
      path: { realm_name: realm },
      query: {
        from_timestamp: range.from,
        to_timestamp: range.to,
        limit: WINDOW_LIMIT,
        event_types: key,
      },
    }).queryOptions,
    enabled: Boolean(realm),
  })

  const events = useMemo(
    () =>
      [...(data?.data ?? [])].sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      ),
    [data]
  )

  return {
    events,
    isLoading,
    isError,
    truncated: events.length >= WINDOW_LIMIT,
    windowDays: WINDOW_DAYS,
    windowLimit: WINDOW_LIMIT,
  }
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
