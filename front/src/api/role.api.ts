import { userStore } from "@/store/user.store"
import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { GetRolesResponse } from "./api.interface"


export const useGetRoles = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["roles"],
    queryFn: async (): Promise<GetRolesResponse> => {
      const access_token = userStore.getState().access_token

      const response = await apiClient.get<GetRolesResponse>(`/realms/${realm}/roles`, {
        headers: {
          Authorization: `Bearer ${access_token}`,
        },
      })

      return response.data
    }
  })
}