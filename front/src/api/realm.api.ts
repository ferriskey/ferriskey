import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { tanstackApi } from '.'

export interface UserRealmsQuery {
  realm: string
}

export const useGetUserRealmsQuery = ({ realm }: UserRealmsQuery) => {
  return useQuery(
    tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useCreateRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...tanstackApi.mutation('post', '/realms', async (response) => {
      const data = await response.json()
      return data
    }).mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['user-realms'],
      })
    },
  })
}
