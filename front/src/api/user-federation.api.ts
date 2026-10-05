import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Endpoints } from './api.client'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type FederationProvidersQuery = NonNullable<
  Endpoints.get_List_providers['parameters']['query']
>

export type FederationProvidersFilter = Omit<
  FederationProvidersQuery,
  'page' | 'limit' | 'order' | 'order_by'
>

export const FEDERATION_PROVIDER_FILTER_KEYS = [
  'search',
  'provider_type',
  'provider_family',
  'enabled',
  'sync_enabled',
  'synced',
  'sync_mode',
  'created_from',
  'created_to',
] as const

export const federationProvidersKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useCreateUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/federation/providers')
      .mutationOptions,
    onSuccess: async (_, params) => {
      const queryKeys = federationProvidersKey(params.path.realm_name)

      await queryClient.invalidateQueries({
        queryKey: queryKeys,
      })
    },
  })
}

export const useGetUserFederations = ({
  realm,
  query,
  keepPrevious = false,
  enabled = true,
}: {
  realm: string
  query?: FederationProvidersQuery
  enabled?: boolean
} & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
      path: { realm_name: realm },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled,
  })
}

export const useFederationProviderCount = ({
  realm,
  filter,
}: {
  realm: string
  filter?: FederationProvidersFilter
}) => {
  const { data, isLoading } = useGetUserFederations({
    realm,
    query: { ...filter, limit: 1 },
  })
  return { count: data?.metadata.total ?? 0, isLoading }
}

export const useGetUserFederation = (realm_name: string, id: string) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/federation/providers/{id}', {
      path: {
        realm_name,
        id,
      },
    }).queryOptions,
    enabled: !!realm_name && !!id,
  })
}

export const useUpdateUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/federation/providers/{id}')
      .mutationOptions,
    onSuccess: async (_, params) => {
      const listQueryKeys = federationProvidersKey(params.path.realm_name)

      const detailQueryKeys = window.tanstackApi.get(
        '/realms/{realm_name}/federation/providers/{id}',
        {
          path: {
            realm_name: params.path.realm_name,
            id: params.path.id,
          },
        }
      ).queryKey

      await queryClient.invalidateQueries({
        queryKey: listQueryKeys,
      })

      await queryClient.invalidateQueries({
        queryKey: detailQueryKeys,
      })
    },
  })
}

export const useDeleteUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/federation/providers/{id}')
      .mutationOptions,
    onSuccess: async (_, params) => {
      const queryKeys = federationProvidersKey(params.path.realm_name)

      await queryClient.invalidateQueries({
        queryKey: queryKeys,
      })
    },
  })
}

export const useTestUserFederationConnection = () => {
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/federation/providers/{id}/test-connection'
    ).mutationOptions,
  })
}

export const useSyncUsers = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/federation/providers/{id}/sync-users'
    ).mutationOptions,
    onSuccess: async (_, params) => {
      const listQueryKeys = federationProvidersKey(params.path.realm_name)

      const detailQueryKeys = window.tanstackApi.get(
        '/realms/{realm_name}/federation/providers/{id}',
        {
          path: {
            realm_name: params.path.realm_name,
            id: params.path.id,
          },
        }
      ).queryKey

      await queryClient.invalidateQueries({
        queryKey: listQueryKeys,
      })

      await queryClient.invalidateQueries({
        queryKey: detailQueryKeys,
      })
    },
  })
}
