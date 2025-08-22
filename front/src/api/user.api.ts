import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
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
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users', async (res) => {
      return res.json()
    }).mutationOptions,
    onSuccess: async (res) => {
      console.log(res)
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
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}', async (res) => {
      return res.json()
    }).mutationOptions,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['user'],
      })
    },
  })
}

export const useBulkDeleteUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/bulk', async (res) => {
      const data = await res.json()
      return data
    }).mutationOptions,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['users'],
      })
    },
  })
}

export const useResetUserPassword = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/users/{user_id}/reset-password',
      async (res) => {
        const data = await res.json()
        return data
      }
    ).mutationOptions,
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
    onSuccess: async (res) => {
      const data = await res.json()
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
