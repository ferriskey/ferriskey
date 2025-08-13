import { useMutation, useQueryClient } from '@tanstack/react-query'
import { tanstackApi } from '@/api/index.ts'

export const useDeleteUserCredential = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...tanstackApi.mutation('delete', '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}')
      .mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["user", "credentials"]
      })
    }
  })
}
