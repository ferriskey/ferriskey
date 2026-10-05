import { useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type SecurityEventsQuery = NonNullable<
  Endpoints.get_Get_security_events['parameters']['query']
>

export type SecurityEventsFilter = Omit<SecurityEventsQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const SECURITY_EVENT_FILTER_KEYS = [
  'search',
  'event_types',
  'status',
  'target_type',
  'actor_id',
] as const

export const useGetSecurityEvents = ({
  realm,
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { query?: SecurityEventsQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/seawatch/v1/security-events', {
      path: { realm_name: realm! },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: !!realm && enabled,
  })
}

export const useSecurityEventCount = ({
  realm,
  filter,
}: BaseQuery & { filter?: SecurityEventsFilter }) => {
  const { data, isLoading } = useGetSecurityEvents({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}
