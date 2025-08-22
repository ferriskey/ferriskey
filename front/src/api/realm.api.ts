import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

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
      return data
    }).mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['user-realms'],
      })
    },
  })
}
