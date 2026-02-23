import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

const GET_POST_LOGOUT_REDIRECT_URIS_PATH =
  '/realms/{realm_name}/clients/{client_id}/post-logout-redirects'
const CREATE_POST_LOGOUT_REDIRECT_URI_PATH =
  '/realms/{realm_name}/clients/{client_id}/post-logout-redirects'
const DELETE_POST_LOGOUT_REDIRECT_URI_PATH =
  '/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}'

export interface GetPostLogoutRedirectUrisQuery {
  realmName?: string
  clientId?: string
}

export const useGetPostLogoutRedirectUris = ({
  realmName,
  clientId,
}: GetPostLogoutRedirectUrisQuery) => {
  return useQuery<Schemas.RedirectUri[]>({
    queryKey: ['post-logout-redirect-uris', realmName, clientId],
    queryFn: async (): Promise<Schemas.RedirectUri[]> => {
      if (!realmName || !clientId) {
        return []
      }

      return (await window.tanstackApi.client.get(GET_POST_LOGOUT_REDIRECT_URIS_PATH as never, {
        path: {
          realm_name: realmName,
          client_id: clientId,
        },
      } as never)) as Schemas.RedirectUri[]
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
    mutationFn: async ({
      realmName,
      clientId,
      payload,
    }: CreatePostLogoutRedirectUriMutate): Promise<Schemas.RedirectUri> => {
      return (await window.tanstackApi.client.post(CREATE_POST_LOGOUT_REDIRECT_URI_PATH as never, {
        path: {
          realm_name: realmName,
          client_id: clientId,
        },
        body: {
          value: payload.value,
          enabled: true,
        },
      } as never)) as Schemas.RedirectUri
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
      return window.tanstackApi.client.delete(DELETE_POST_LOGOUT_REDIRECT_URI_PATH as never, {
        path: {
          realm_name: realmName,
          client_id: clientId,
          uri_id: redirectUriId,
        },
      } as never)
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['post-logout-redirect-uris', variables.realmName, variables.clientId],
      })
    },
  })
}
