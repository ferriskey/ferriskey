import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { GetRolesResponse } from "./api.interface"
import { authStore } from "@/store/auth.store"


export const useGetRoles = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["roles"],
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