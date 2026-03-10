import { useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'

export const useGetFlows = ({
  realm,
  clientId,
  userId,
  grantType,
  status,
  limit,
  offset,
}: BaseQuery & {
  clientId?: string
  userId?: string
  grantType?: string
  status?: string
  limit?: number
  offset?: number
}) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/compass/v1/flows', {
      path: { realm_name: realm! },
      query: {
        client_id: clientId,
        user_id: userId,
        grant_type: grantType,
        status,
        limit,
        offset,
      },
    }).queryOptions,
    enabled: !!realm,
  })
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
