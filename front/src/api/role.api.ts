import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { GetRoleResponse, GetRolesResponse, Role } from "./api.interface"
import { authStore } from "@/store/auth.store"


export const useGetRoles = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["roles", realm],
    queryFn: async (): Promise<GetRolesResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<GetRolesResponse>(`/realms/${realm}/roles`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    }
  })
}

export const useGetRole = ({ realm, roleId }: BaseQuery & { roleId?: string }) => {
  return useQuery({
    queryKey: ["roles", realm, roleId],
    queryFn: async (): Promise<Role> => {
      const accessToken = authStore.getState().accessToken

      const { data: response } = await apiClient.get<GetRoleResponse>(`/realms/${realm}/roles/${roleId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    enabled: !!realm && !!roleId,
  })
}