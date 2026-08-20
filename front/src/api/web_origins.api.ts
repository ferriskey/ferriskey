import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

export const webOriginsQueryKey = (realmName: string, clientId: string) => [
  'web-origins',
  realmName,
  clientId,
]

export const useGetWebOrigins = ({
  realmName,
  clientId,
}: {
  realmName?: string
  clientId?: string
}) => {
  return useQuery({
    queryKey: webOriginsQueryKey(realmName ?? '', clientId ?? ''),
    queryFn: async () =>
      window.tanstackApi.client.get('/realms/{realm_name}/clients/{client_id}/web-origins', {
        path: { realm_name: realmName!, client_id: clientId! },
      }) as Promise<Schemas.WebOrigin[]>,
    enabled: !!realmName && !!clientId,
  })
}

export interface CreateWebOriginMutate {
  realmName: string
  clientId: string
  payload: {
    value: string
  }
}

export const useCreateWebOrigin = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: CreateWebOriginMutate) => {
      return window.tanstackApi.client.post(
        '/realms/{realm_name}/clients/{client_id}/web-origins',
        {
          path: { realm_name: realmName, client_id: clientId },
          body: { value: payload.value },
        }
      ) as Promise<Schemas.WebOrigin>
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: webOriginsQueryKey(variables.realmName, variables.clientId),
      })
    },
  })
}

export interface DeleteWebOriginMutate {
  realmName: string
  clientId: string
  webOriginId: string
}

export const useDeleteWebOrigin = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, clientId, webOriginId }: DeleteWebOriginMutate) => {
      return window.tanstackApi.client.delete(
        '/realms/{realm_name}/clients/{client_id}/web-origins/{web_origin_id}',
        {
          path: {
            realm_name: realmName,
            client_id: clientId,
            web_origin_id: webOriginId,
          },
        }
      )
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: webOriginsQueryKey(variables.realmName, variables.clientId),
      })
    },
  })
}
