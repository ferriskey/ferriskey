import { CreateClientSchema } from '@/pages/iam/client/schemas/create-client.schema.ts'
import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQueries, useQuery, useQueryClient, type UseQueryResult } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Endpoints, Schemas } from './api.client'
import { ID_BATCH, idBatches } from './id-batches'
import { clientScopesKey } from './client-scope.api'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type ClientsQuery = NonNullable<Endpoints.get_Get_clients['parameters']['query']>

export type ClientsFilter = Omit<ClientsQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const CLIENT_SEARCH_LIMIT = 20

export const APPLICATION_FILTER_KEYS = [
  'search',
  'name',
  'enabled',
  'application_type',
  'created_from',
  'created_to',
] as const

export const CLIENT_FILTER_KEYS = [
  'search',
  'name',
  'enabled',
  'public_client',
  'service_account_enabled',
  'protocol',
  'client_type',
  'has_redirect_uris',
  'maintenance_enabled',
  'created_from',
  'created_to',
] as const

const SEARCH_DEBOUNCE_MS = 300

const clientsKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/clients', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetClients = ({
  realm,
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { query?: ClientsQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients', {
      path: {
        realm_name: realm || 'master',
      },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled,
  })
}

export const useClientCount = ({ realm, filter }: BaseQuery & { filter?: ClientsFilter }) => {
  const { data, isLoading } = useGetClients({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}

const combineClients = (results: UseQueryResult<Schemas.Paginated_Client>[]) => ({
  clients: results.flatMap((result) => result.data?.data ?? []),
  isLoading: results.some((result) => result.isLoading),
})

export const useClientsByIds = ({ realm, ids }: BaseQuery & { ids: readonly string[] }) => {
  const batches = useMemo(() => idBatches(ids), [ids])
  return useQueries({
    queries: batches.map((batch) => ({
      ...window.tanstackApi.get('/realms/{realm_name}/clients', {
        path: { realm_name: realm || 'master' },
        query: { ids: batch, limit: ID_BATCH },
      }).queryOptions,
    })),
    combine: combineClients,
  })
}

export const useClientSearch = ({
  realm,
  enabled = true,
}: BaseQuery & { enabled?: boolean }) => {
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const { data, isLoading } = useGetClients({
    realm,
    query: {
      search: debounced || undefined,
      order_by: 'name',
      order: 'asc',
      limit: CLIENT_SEARCH_LIMIT,
    },
    enabled,
  })

  return {
    search,
    setSearch,
    clients: data?.data ?? [],
    isLoading,
  }
}

export const useGetClient = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
      path: {
        client_id: clientId!,
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export const useGetClientSecret = ({
  realm,
  clientId,
  enabled,
}: BaseQuery & { clientId?: string; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}/client-secret', {
      path: {
        realm_name: realm!,
        client_id: clientId!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm && !!enabled,
    gcTime: 0,
    staleTime: Infinity,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  })
}

export interface CreateClientMutate {
  realm: string
  payload: CreateClientSchema
}

export const useCreateClient = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/clients').mutationOptions,
    onSuccess: async (_, variables) => {
      await queryClient.invalidateQueries({
        queryKey: clientsKey(variables.path.realm_name),
      })
    },
  })
}

export const useUpdateClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('patch', '/realms/{realm_name}/clients/{client_id}')
      .mutationOptions,
    onSuccess: async (payload, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
        path: {
          client_id: variables.path.client_id,
          realm_name: variables.path.realm_name,
        },
      }).queryKey

      toast.success(translate('common:toast.client.updated', { name: payload.data.name }))
      queryClient.invalidateQueries({
        queryKey: keys,
      })
      queryClient.invalidateQueries({
        queryKey: clientsKey(variables.path.realm_name),
      })
    },
  })
}

export const useDeleteClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/clients/{client_id}')
      .mutationOptions,
    onSuccess: async (res) => {
      await queryClient.invalidateQueries({
        queryKey: clientsKey(res.realm_name),
      })
    },
  })
}

export const useGetClientRoles = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}/roles', {
      path: {
        realm_name: realm!,
        client_id: clientId!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export interface EvaluateClientScopesParams {
  realm: string
  clientId: string
  userId: string
  scope?: string
}

export const useEvaluateClientScopes = () => {
  return useMutation({
    mutationFn: async ({ realm, clientId, userId, scope }: EvaluateClientScopesParams) =>
      window.tanstackApi
        .mutation('post', '/realms/{realm_name}/clients/{client_id}/evaluate-scopes')
        .mutationOptions.mutationFn({
          path: {
            realm_name: realm,
            client_id: clientId,
          },
          body: {
            user_id: userId,
            scope,
          },
        }),
  })
}

export const useGetClientScopes = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}/client-scopes', {
      path: {
        realm_name: realm!,
        client_id: clientId!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

// Unified scope assignment interface
export interface AssignScopeParams {
  realm: string
  clientId: string
  scopeId: string
  type: 'default' | 'optional'
}

export const useAssignScope = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, clientId, scopeId, type }: AssignScopeParams) => {
      const endpoint =
        type === 'default'
          ? '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}'
          : '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}'

      return window.tanstackApi.mutation('put', endpoint).mutationOptions.mutationFn({
        path: {
          realm_name: realm,
          client_id: clientId,
          scope_id: scopeId,
        },
      })
    },
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get(
        '/realms/{realm_name}/clients/{client_id}/client-scopes',
        {
          path: {
            realm_name: variables.realm,
            client_id: variables.clientId,
          },
        }
      )
      await Promise.all([
        queryClient.invalidateQueries({ queryKey }),
        queryClient.invalidateQueries({ queryKey: clientScopesKey(variables.realm) }),
      ])
      toast.success(translate(`common:toast.client.scope_assigned.${variables.type}`))
    },
    onError: (error) => {
      toast.error(translate('common:toast.client.scope_assign_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useUnassignScope = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, clientId, scopeId, type }: AssignScopeParams) => {
      const endpoint =
        type === 'default'
          ? '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}'
          : '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}'

      return window.tanstackApi.mutation('delete', endpoint).mutationOptions.mutationFn({
        path: {
          realm_name: realm,
          client_id: clientId,
          scope_id: scopeId,
        },
      })
    },
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get(
        '/realms/{realm_name}/clients/{client_id}/client-scopes',
        {
          path: {
            realm_name: variables.realm,
            client_id: variables.clientId,
          },
        }
      )
      await Promise.all([
        queryClient.invalidateQueries({ queryKey }),
        queryClient.invalidateQueries({ queryKey: clientScopesKey(variables.realm) }),
      ])
      toast.success(translate(`common:toast.client.scope_unassigned.${variables.type}`))
    },
    onError: (error) => {
      toast.error(translate('common:toast.client.scope_unassign_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}
