import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetRoles = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/roles', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useGetRole = ({ realm, roleId }: BaseQuery & { roleId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/roles/{role_id}', {
      path: {
        realm_name: realm!,
        role_id: roleId!,
      },
    }).queryOptions,
    staleTime: 5 * 60 * 1000,
    enabled: !!realm && !!roleId,
  })
}

export const useCreateRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/clients/{client_id}/roles',
      async (res) => {
        return res.json()
      }
    ).mutationOptions,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['roles'] })
    },
  })
}

export const useUpdateRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/roles/{role_id}', async (res) =>
      res.json()
    ).mutationOptions,
    onSuccess(res) {
      queryClient.invalidateQueries({ queryKey: ['role', res.data.id] })
      toast.success('Role updated successfully', {
        description: `Role ${res.data.name} has been updated successfully.`,
      })
    },
    onError(error) {
      toast.error('Failed to update role', {
        description: error.message,
      })
    },
  })
}

export const useUpdateRolePermissions = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'patch',
      '/realms/{realm_name}/roles/{role_id}/permissions',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess(res) {
      queryClient.invalidateQueries({ queryKey: ['role', res.data.id] })
      toast.success('Role permissions updated successfully', {
        description: `Role ${res.data.name} permissions has been updated successfully.`,
      })
    },
    onError(error) {
      toast.error('Failed to update role', {
        description: error.message,
      })
    },
  })
}

export const useDeleteRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/roles/{role_id}',
      async (res) => res.json()
    ).mutationOptions,
    // FIXME: there is no bulk delete endpoint, and this one may be inefficient, and the
    // stacked toast messages will look bad.
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/roles', {
        path: {
          realm_name: variables.path.realm_name,
        },
      })
      await queryClient.invalidateQueries({
        queryKey: [...queryKey],
      })
      toast.success('Role deleted successfully', {
        description: 'Role has been deleted from client successfully.',
      })
    },
    onError(error) {
      toast.error('Failed to delete role', {
        description: error.message,
      })
    },
  })
}
