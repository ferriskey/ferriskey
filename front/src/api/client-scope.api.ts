import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

type ProtocolMapperQuery = BaseQuery & { scopeId: string }

export type ClientScopesQuery = NonNullable<
  Endpoints.get_Get_client_scopes['parameters']['query']
>

export type ClientScopesFilter = Omit<ClientScopesQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const CLIENT_SCOPE_FILTER_KEYS = [
  'name',
  'description',
  'protocol',
  'default_scope_type',
  'has_protocol_mappers',
] as const

export const CLIENT_SCOPE_SEARCH_LIMIT = 20

const SEARCH_DEBOUNCE_MS = 300

const clientScopesKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetClientScopes = ({
  realm = 'master',
  query,
  enabled = true,
}: BaseQuery & { query?: ClientScopesQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
      path: {
        realm_name: realm,
      },
      query: query ?? {},
    }).queryOptions,
    enabled,
  })
}

export const useClientScopeCount = ({
  realm,
  filter,
  enabled = true,
}: BaseQuery & { filter?: ClientScopesFilter; enabled?: boolean }) => {
  const { data, isLoading } = useGetClientScopes({
    realm,
    query: { ...filter, limit: 1 },
    enabled,
  })
  return { count: data?.metadata.total ?? 0, isLoading }
}

export const useClientScopeSearch = ({
  realm,
  enabled = true,
}: BaseQuery & { enabled?: boolean }) => {
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const { data, isLoading } = useGetClientScopes({
    realm,
    query: {
      search: debounced || undefined,
      order_by: 'name',
      order: 'asc',
      limit: CLIENT_SCOPE_SEARCH_LIMIT,
    },
    enabled,
  })

  return {
    search,
    setSearch,
    scopes: data?.data ?? [],
    isLoading,
  }
}

export const useGetClientScope = ({ realm, scopeId }: BaseQuery & { scopeId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
      path: {
        realm_name: realm!,
        scope_id: scopeId!,
      },
    }).queryOptions,
    enabled: !!realm && !!scopeId,
  })
}

export const useCreateClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/client-scopes').mutationOptions,
    onSuccess: async (_, variables) => {
      await queryClient.invalidateQueries({
        queryKey: clientScopesKey(variables.path.realm_name),
      })
      toast.success(translate('common:toast.client_scope.created'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.client_scope.create_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useCreateProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success(translate('common:toast.protocol_mapper.created'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.protocol_mapper.create_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useUpdateProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'patch',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success(translate('common:toast.protocol_mapper.updated'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.protocol_mapper.update_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success(translate('common:toast.protocol_mapper.deleted'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.protocol_mapper.delete_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useUpdateClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('patch', '/realms/{realm_name}/client-scopes/{scope_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey: scopeKey } = window.tanstackApi.get(
        '/realms/{realm_name}/client-scopes/{scope_id}',
        {
          path: {
            realm_name: variables.path.realm_name,
            scope_id: variables.path.scope_id,
          },
        }
      )
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: scopeKey }),
        queryClient.invalidateQueries({ queryKey: clientScopesKey(variables.path.realm_name) }),
      ])
      toast.success(translate('common:toast.client_scope.updated'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.client_scope.update_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/client-scopes/{scope_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      await queryClient.invalidateQueries({
        queryKey: clientScopesKey(variables.path.realm_name),
      })
      toast.success(translate('common:toast.client_scope.deleted'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.client_scope.delete_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

// Re-export type for use in feature files
export type { ProtocolMapperQuery }
