import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import { toast } from 'sonner'

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
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/client-scopes', async (res) =>
      res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: {
          realm_name: variables.path.realm_name,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Client scope created successfully')
    },
    onError: (error) => {
      toast.error('Failed to create client scope', {
        description: error.message,
      })
    },
  })
}

export const useCreateProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Protocol mapper created successfully')
    },
    onError: (error) => {
      toast.error('Failed to create protocol mapper', { description: error.message })
    },
  })
}

export const useUpdateProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'patch',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Protocol mapper updated successfully')
    },
    onError: (error) => {
      toast.error('Failed to update protocol mapper', { description: error.message })
    },
  })
}

export const useDeleteProtocolMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
        path: {
          realm_name: variables.path.realm_name,
          scope_id: variables.path.scope_id,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Protocol mapper deleted successfully')
    },
    onError: (error) => {
      toast.error('Failed to delete protocol mapper', { description: error.message })
    },
  })
}

export const useUpdateClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'patch',
      '/realms/{realm_name}/client-scopes/{scope_id}',
      async (res) => res.json()
    ).mutationOptions,
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
      toast.success('Client scope updated successfully')
    },
    onError: (error) => {
      toast.error('Failed to update client scope', { description: error.message })
    },
  })
}

export const useDeleteClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/client-scopes/{scope_id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: { realm_name: variables.path.realm_name },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Client scope deleted successfully')
    },
    onError: (error) => {
      toast.error('Failed to delete client scope', { description: error.message })
    },
  })
}

// Re-export type for use in feature files
export type { ProtocolMapperQuery }
