import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'

export interface UserRealmsQuery {
  realm: string
}

export const useGetUserRealmsQuery = ({ realm }: UserRealmsQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useCreateRealm = ({ realm }: UserRealmsQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms').mutationOptions,

    onSuccess: async () => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
        path: {
          realm_name: realm,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useGetLoginSettings = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{name}/login-settings', {
      path: {
        name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetRealm = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{name}', {
      path: {
        name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useDeleteRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{name}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
        path: {
          realm_name: variables.path.name,
        },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useUpdateRealmSettings = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{name}/settings').mutationOptions,
    onSuccess: async (res) => {
      const queryKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: {
          name: res.data.name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: [...queryKeys],
      })
    },
  })
}

export const useGetPasswordPolicy = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/password-policy', {
      path: {
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdatePasswordPolicy = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/password-policy').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/password-policy', {
        path: {
          realm_name: variables.path.realm_name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}
