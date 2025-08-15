import { authStore } from '@/store/auth.store'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '.'
import { RedirectUri } from './core.interface'

export interface CreateRedirectUriMutate {
  realmName: string,
  clientId: string,
  payload: {
    value: string
  }
}

export const useCreateRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: CreateRedirectUriMutate) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.post<RedirectUri>(`/realms/${realmName}/clients/${clientId}/redirects`, {
        value: payload.value,
        enabled: true,
      }, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['client']
      })
    }
  })
}

export interface DeleteRedirectUriMutate {
  realmName: string,
  clientId: string,
  redirectUriId: string
}

export const useDeleteRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, redirectUriId }: DeleteRedirectUriMutate) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.delete(`/realms/${realmName}/clients/${clientId}/redirects/${redirectUriId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['client']
      })
    }
  })
}
