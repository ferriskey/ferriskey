import { userStore } from "@/store/user.store"
import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { RolesResponse } from "./api.interface"


export const useGetRoles = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["roles"],
    queryFn: async (): Promise<RolesResponse> => {
      const access_token = userStore.getState().access_token

      const response = await apiClient.get<RolesResponse>(`/realms/${realm}/roles`, {
        headers: {
          Authorization: `Bearer ${access_token}`,
        },
      })

      return response.data
    }
  })
}