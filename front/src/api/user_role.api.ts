import { useMutation, useQueryClient } from '@tanstack/react-query'
import { tanstackApi } from '.'

export const useUnassignUserRole = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/users/{user_id}/roles/{role_id}',
      async (response) => {
        const data = await response.json()
        return data
      }
    ).mutationOptions,
    onSuccess: async (res) => {
      const keys = tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
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
