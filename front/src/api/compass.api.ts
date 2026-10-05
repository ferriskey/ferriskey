import { useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type FlowsQuery = NonNullable<Endpoints.get_Get_flows['parameters']['query']>

export type FlowsFilter = Omit<FlowsQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const FLOW_FILTER_KEYS = [
  'search',
  'status',
  'grant_type',
  'client_id',
  'user_id',
  'identified',
  'completed',
] as const

export const useGetFlows = ({
  realm,
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { query?: FlowsQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/compass/v1/flows', {
      path: { realm_name: realm! },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: !!realm && enabled,
  })
}

export const useFlowCount = ({ realm, filter }: BaseQuery & { filter?: FlowsFilter }) => {
  const { data, isLoading } = useGetFlows({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}

export const useGetFlow = ({ realm, flowId }: BaseQuery & { flowId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/compass/v1/flows/{flow_id}', {
      path: { realm_name: realm!, flow_id: flowId },
    }).queryOptions,
    enabled: !!realm && !!flowId,
  })
}

export const useGetStats = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/compass/v1/stats', {
      path: { realm_name: realm! },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetDailyActivityStats = ({
  realm,
  from,
  to,
  clientId,
  userId,
  grantType,
}: BaseQuery & {
  from?: string
  to?: string
  clientId?: string
  userId?: string
  grantType?: string
}) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/compass/v1/activity/daily', {
      path: { realm_name: realm! },
      query: {
        from,
        to,
        client_id: clientId,
        user_id: userId,
        grant_type: grantType,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}
