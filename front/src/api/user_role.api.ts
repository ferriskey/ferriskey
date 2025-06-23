import { useMutation, useQueryClient } from "@tanstack/react-query"
import { UserMutateContract} from './user.api'
import { authStore } from "@/store/auth.store"
import { apiClient } from "."
import { UnassignRoleResponse } from "./api.interface"
export const useUnassignUserRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realm, userId, payload }: UserMutateContract<{ roleId: string }>): Promise<UnassignRoleResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.delete<UnassignRoleResponse>(`/realms/${realm}/users/${userId}/roles/${payload.roleId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        }
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user-roles"],
      })
    }
  })
}