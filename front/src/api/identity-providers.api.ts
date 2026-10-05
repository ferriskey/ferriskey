import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Endpoints } from './api.client'

export interface CreateProviderInput {
  alias: string
  provider_id: string
  enabled: boolean
  display_name?: string | null
  first_broker_login_flow_alias?: string | null
  post_broker_login_flow_alias?: string | null
  store_token?: boolean
  add_read_token_role_on_create?: boolean
  trust_email?: boolean
  link_only?: boolean
  config: Record<string, unknown>
}

export interface UpdateProviderInput {
  display_name?: string
  enabled?: boolean
  first_broker_login_flow_alias?: string | null
  post_broker_login_flow_alias?: string | null
  store_token?: boolean
  add_read_token_role_on_create?: boolean
  trust_email?: boolean
  link_only?: boolean
  config?: Record<string, unknown>
}

// TanStack Query hooks
interface BaseQuery {
  realm: string
}

interface ProviderQuery extends BaseQuery {
  providerId: string
}

export type IdentityProvidersQuery = NonNullable<
  Endpoints.get_List_identity_providers['parameters']['query']
>

export type IdentityProvidersFilter = Omit<
  IdentityProvidersQuery,
  'page' | 'limit' | 'order' | 'order_by'
>

export const IDENTITY_PROVIDER_FILTER_KEYS = [
  'search',
  'alias',
  'display_name',
  'provider_id',
  'enabled',
  'health',
  'created_from',
  'created_to',
] as const

export const identityProvidersKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetIdentityProviders = ({
  realm,
  query,
  enabled = true,
}: BaseQuery & { query?: IdentityProvidersQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
      path: { realm_name: realm ?? 'master' },
      query: query ?? {},
    }).queryOptions,
    enabled,
  })
}

export const useIdentityProviderCount = ({
  realm,
  filter,
}: BaseQuery & { filter?: IdentityProvidersFilter }) => {
  const { data, isLoading } = useGetIdentityProviders({
    realm,
    query: { ...filter, limit: 1 },
  })
  return { count: data?.metadata.total ?? 0, isLoading }
}

export const useCreateIdentityProvider = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/identity-providers')
      .mutationOptions,
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: identityProvidersKey(variables.path.realm_name) })
    },
  })
}

export const useIdentityProvider = ({ realm, providerId }: ProviderQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/identity-providers/{alias}', {
      path: {
        realm_name: realm ?? 'master',
        alias: providerId,
      },
    }).queryOptions,
    enabled: !!realm && !!providerId,
  })
}

export const useUpdateIdentityProvider = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/identity-providers/{alias}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = identityProvidersKey(variables.path.realm_name)

      const detailKeys = window.tanstackApi.get('/realms/{realm_name}/identity-providers/{alias}', {
        path: {
          realm_name: variables.path.realm_name,
          alias: variables.path.alias,
        },
      }).queryOptions.queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      await queryClient.invalidateQueries({ queryKey: detailKeys })
    },
  })
}

export const useDeleteIdentityProvider = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/identity-providers/{alias}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = identityProvidersKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
