import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

type ProtocolMapperQuery = BaseQuery & { scopeId: string }

export const useGetClientScopes = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
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
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: {
          realm_name: variables.path.realm_name,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
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
      const { queryKey: listKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: { realm_name: variables.path.realm_name },
      })
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: scopeKey }),
        queryClient.invalidateQueries({ queryKey: listKey }),
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
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: { realm_name: variables.path.realm_name },
      })
      await queryClient.invalidateQueries({ queryKey })
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
