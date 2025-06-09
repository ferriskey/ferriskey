import { useMutation, useQueryClient } from '@tanstack/react-query'
import { authStore } from '@/store/auth.store.ts'
import { apiClient } from '@/api/index.ts'
import { DeleteUserCredentialResponse } from '@/api/api.interface.ts'

export interface DeleteUserCredentialParams {
  realm: string
  userId: string
  credentialId: string
}

export const useDeleteUserCredential = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realm, userId, credentialId }: DeleteUserCredentialParams): Promise<DeleteUserCredentialResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.delete<DeleteUserCredentialResponse>(`/realms/${realm}/users/${userId}/credentials/${credentialId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`
        }
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user", "credentials"]
      })
    }
  })
}
