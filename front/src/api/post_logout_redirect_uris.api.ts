import { authStore } from '@/store/auth.store'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { RedirectUri } from './core.interface'

export interface GetPostLogoutRedirectUrisQuery {
  realmName?: string
  clientId?: string
}

export const useGetPostLogoutRedirectUris = ({
  realmName,
  clientId,
}: GetPostLogoutRedirectUrisQuery) => {
  return useQuery({
    queryKey: ['post-logout-redirect-uris', realmName, clientId],
    queryFn: async (): Promise<RedirectUri[]> => {
      const accessToken = authStore.getState().accessToken

      const response = await window.axios.get<RedirectUri[]>(
        `/realms/${realmName}/clients/${clientId}/post-logout-redirects`,
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        }
      )

      return response.data
    },
    enabled: !!realmName && !!clientId,
  })
}

export interface CreatePostLogoutRedirectUriMutate {
  realmName: string
  clientId: string
  payload: {
    value: string
  }
}

export const useCreatePostLogoutRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: CreatePostLogoutRedirectUriMutate) => {
      const accessToken = authStore.getState().accessToken

      const response = await window.axios.post<RedirectUri>(
        `/realms/${realmName}/clients/${clientId}/post-logout-redirects`,
        {
          value: payload.value,
          enabled: true,
        },
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        }
      )

      return response.data
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['post-logout-redirect-uris', variables.realmName, variables.clientId],
      })
    },
  })
}

export interface DeletePostLogoutRedirectUriMutate {
  realmName: string
  clientId: string
  redirectUriId: string
}

export const useDeletePostLogoutRedirectUri = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realmName, clientId, redirectUriId }: DeletePostLogoutRedirectUriMutate) => {
      const accessToken = authStore.getState().accessToken

      const response = await window.axios.delete(
        `/realms/${realmName}/clients/${clientId}/post-logout-redirects/${redirectUriId}`,
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        }
      )

      return response.data
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['post-logout-redirect-uris', variables.realmName, variables.clientId],
      })
    },
  })
}
