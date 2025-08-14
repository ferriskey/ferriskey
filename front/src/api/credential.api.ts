import { useMutation, useQueryClient } from '@tanstack/react-query'
import { tanstackApi } from '@/api/index.ts'

export const useDeleteUserCredential = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}',
      async (res) => {
        return res.json()
      }
    ).mutationOptions,
    onSuccess: async (payload) => {
      const keys = tanstackApi.get('/realms/{realm_name}/users/{user_id}/credentials', {
        path: {
          realm_name: payload.realm_name,
          user_id: payload.user_id,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}
