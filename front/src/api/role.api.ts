import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { GetRoleResponse, GetRolesResponse, Role } from "./api.interface"
import { authStore } from "@/store/auth.store"
import { CreateRoleSchema } from "@/pages/role/schemas/create-role.schema"


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
    staleTime: 5 * 60 * 1000,
    enabled: !!realm && !!roleId,
  })
}

export const useCreateRole = () => {
  const queryClient = useQueryClient()
  
  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: { realmName: string, clientId: string, payload: CreateRoleSchema }) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.post(`/realms/${realmName}/clients/${clientId}/roles`, payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["roles"] })
    }
  })
}
