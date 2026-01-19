import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

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

export const useGetIdentityProviders = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
      path: { realm_name: realm ?? 'master', brief_representation: null },
    }).queryOptions,
  })
}

export const useCreateIdentityProvider = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/identity-providers')
      .mutationOptions,
    onSuccess: (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
        path: {
          realm_name: variables.path.realm_name,
          brief_representation: null,
        },
      }).queryOptions.queryKey

      queryClient.invalidateQueries({ queryKey: keys })
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
      const keys = window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
        path: {
          realm_name: variables.path.realm_name,
          brief_representation: null,
        },
      }).queryOptions.queryKey

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
      const keys = window.tanstackApi.get('/realms/{realm_name}/identity-providers', {
        path: {
          realm_name: variables.path.realm_name,
          brief_representation: null,
        },
      }).queryOptions.queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
