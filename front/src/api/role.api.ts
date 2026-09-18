import { QueryClient, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

const invalidateRole = async (
  queryClient: QueryClient,
  realmName: string,
  roleId: string
) => {
  const role = window.tanstackApi.get('/realms/{realm_name}/roles/{role_id}', {
    path: { realm_name: realmName, role_id: roleId },
  })
  const roles = window.tanstackApi.get('/realms/{realm_name}/roles', {
    path: { realm_name: realmName },
  })

  await Promise.all([
    queryClient.invalidateQueries({ queryKey: role.queryKey }),
    queryClient.invalidateQueries({ queryKey: roles.queryKey }),
  ])
}

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
  const createRealmRole = window.tanstackApi.mutation('post', '/realms/{realm_name}/roles')
  const createClientRole = window.tanstackApi.mutation(
    'post',
    '/realms/{realm_name}/clients/{client_id}/roles'
  )

  return useMutation({
    mutationFn: async ({
      realmName,
      clientId,
      body,
    }: {
      realmName: string
      clientId?: string
      body: Schemas.CreateRoleValidator
    }) => {
      if (clientId) {
        return createClientRole.mutationOptions.mutationFn({
          path: {
            realm_name: realmName,
            client_id: clientId,
          },
          body,
        })
      }

      return createRealmRole.mutationOptions.mutationFn({
        path: {
          realm_name: realmName,
        },
        body,
      })
    },
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/roles', {
        path: {
          realm_name: variables.realmName,
        },
      })
      await queryClient.invalidateQueries({ queryKey })

      if (variables.clientId) {
        const clientRolesQuery = window.tanstackApi.get(
          '/realms/{realm_name}/clients/{client_id}/roles',
          {
            path: {
              realm_name: variables.realmName,
              client_id: variables.clientId,
            },
          }
        )

        await queryClient.invalidateQueries({ queryKey: clientRolesQuery.queryKey })
      }

      toast.success(translate('common:toast.role.created'))
    },
    onError(error) {
      toast.error(translate('common:toast.role.create_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useUpdateRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/roles/{role_id}').mutationOptions,
    async onSuccess(res, variables) {
      await invalidateRole(queryClient, variables.path.realm_name, variables.path.role_id)
      toast.success(translate('common:toast.role.updated'), {
        description: translate('common:toast.role.updated_detail', { name: res.data.name }),
      })
    },
    onError(error) {
      toast.error(translate('common:toast.role.update_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useUpdateRolePermissions = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('patch', '/realms/{realm_name}/roles/{role_id}/permissions')
      .mutationOptions,
    async onSuccess(res, variables) {
      await invalidateRole(queryClient, variables.path.realm_name, variables.path.role_id)
      toast.success(translate('common:toast.role.permissions_updated'), {
        description: translate('common:toast.role.permissions_updated_detail', {
          name: res.data.name,
        }),
      })
    },
    onError(error) {
      toast.error(translate('common:toast.role.update_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/roles/{role_id}')
      .mutationOptions,
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
      toast.success(translate('common:toast.role.deleted'), {
        description: translate('common:toast.role.deleted_detail'),
      })
    },
    onError(error) {
      toast.error(translate('common:toast.role.delete_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}
