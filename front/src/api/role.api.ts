import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { GetRoleResponse, GetRolesResponse, Role, UpdateRoleResponse } from "./api.interface"
import { authStore } from "@/store/auth.store"
import { CreateRoleSchema } from "@/pages/role/schemas/create-role.schema"
import { UpdateRoleSchema } from "@/pages/role/schemas/update-role.schema"
import { toast } from "sonner"


export const useGetRoles = ({ realm = 'master' }: BaseQuery) => {
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
    queryKey: ["role", roleId],
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

export const useUpdateRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, roleId, payload }: { realmName: string, roleId: string, payload: UpdateRoleSchema }): Promise<Role> => {

      const accessToken = authStore.getState().accessToken

      const { data: response } = await apiClient.put<UpdateRoleResponse>(`/realms/${realmName}/roles/${roleId}`, payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess(data) {
      queryClient.invalidateQueries({
        queryKey: ["role", data.id]
      })

      toast.success("Role updated successfully", {
        description: `Role ${data.name} has been updated successfully.`,
      })
    },
    onError(error) {
      toast.error("Failed to update role", {
        description: error.message,
      })
    },
  })
}