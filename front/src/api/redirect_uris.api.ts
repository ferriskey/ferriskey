import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

export interface CreateRedirectUriMutate {
  realmName: string
  clientId: string
  payload: {
    value: string
  }
}

export const useCreateRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: CreateRedirectUriMutate) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/clients/{client_id}/redirects', {
        path: { realm_name: realmName, client_id: clientId },
        body: {
          value: payload.value,
          enabled: true,
        },
      }) as Promise<Schemas.RedirectUri>
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['client'],
      })
    },
  })
}

export interface DeleteRedirectUriMutate {
  realmName: string
  clientId: string
  redirectUriId: string
}

export const useDeleteRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, redirectUriId }: DeleteRedirectUriMutate) => {
      return window.tanstackApi.client.delete('/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}', {
        path: { realm_name: realmName, client_id: clientId, uri_id: redirectUriId },
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['client'],
      })
    },
  })
}
