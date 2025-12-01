import { toast } from 'sonner'
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
    ...window.tanstackApi.mutation('post', '/realms', async (res) => {
      return await res.json()
    }).mutationOptions,

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

    onError: (error: Error) => {
      toast.error(error.message)
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

export const useUpdateRealmSettings = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{name}/settings', async (res) => {
      return res.json()
    }).mutationOptions,
    onSuccess: async (res) => {
      const queryKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: {
          name: res.name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: [...queryKeys],
      })
    },
  })
}
