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
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['user-roles'],
      })
    },
  })
}
