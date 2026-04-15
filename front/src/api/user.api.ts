import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export interface UserMutateContract<T> {
  realm?: string
  userId?: string
  payload: T
}

export interface GetUserQueryParams {
  realm?: string
  userId?: string
}

export const useGetUsers = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users', {
      path: {
        realm_name: realm || 'master',
      },
    }).queryOptions,
  })
}

export const useGetUser = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}

export const useGetUserCredentials = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/credentials', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}

export const useCreateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users').mutationOptions,
    onSuccess: async (res) => {
      const queryKeys = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name: res.data.realm!.name,
        },
      }).queryKey

      console.log(queryKeys)
      await queryClient.invalidateQueries({
        queryKey: [...queryKeys],
      })
    },
  })
}

export const useUpdateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}').mutationOptions,
    onSuccess: (_res, variables) => {
      const { realm_name, user_id } = variables.path
      const userDetailKey = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}', {
        path: {
          realm_name,
          user_id,
        },
      }).queryKey
      const usersListKey = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name,
        },
      }).queryKey
      queryClient.invalidateQueries({ queryKey: userDetailKey })
      queryClient.invalidateQueries({ queryKey: usersListKey })
    },
  })
}

export const useBulkDeleteUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/bulk').mutationOptions,
    onSuccess: async (res) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name: res.realm_name,
        },
      }).queryKey
      queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useResetUserPassword = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}/reset-password')
      .mutationOptions,
    onSuccess: async (res) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/credentials', {
        path: {
          realm_name: res.realm_name,
          user_id: res.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useGetUserRoles = ({ realm, userId }: BaseQuery & { userId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!realm && !!userId,
  })
}

export const useAssignUserRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/{user_id}/roles/{role_id}')
      .mutationOptions,
    onSuccess: async (data) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
        path: {
          realm_name: data.realm_name,
          user_id: data.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

// ─── User Attributes ──────────────────────────────────────────────────────────

export const useGetUserAttributes = ({
  realm,
  userId,
}: BaseQuery & { userId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
      path: { realm_name: realm!, user_id: userId! },
    }).queryOptions,
    select: (response) => response.data,
    enabled: !!realm && !!userId,
  })
}

export const useSetUserAttributes = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}/attributes')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useDeleteUserAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/users/{user_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      toast.success('Attribute deleted')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
