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

export const useCreateRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms', async (response) => {
      const data = await response.json()
      if (!response.ok) {
        throw new Error(
          (data as unknown as { message: string }).message || 'Failed to create realm'
        )
      }
      return data
    }).mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        predicate: (query) => {
          const queryKey = query.queryKey[0] as { _id?: string }
          return queryKey._id === '/realms/{realm_name}/users/@me/realms'
        },
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
