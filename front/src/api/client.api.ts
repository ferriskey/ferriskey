import { userStore } from "@/store/user.store"
import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { ClientsResponse } from "./api.interface"


export const useGetClients = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["clients"],
    queryFn: async (): Promise<ClientsResponse> => {
      const access_token = userStore.getState().access_token

      const response = await apiClient.get<ClientsResponse>(`/realms/${realm}/clients`, {
        headers: {
          Authorization: `Bearer ${access_token}`,
        },
      })

      return response.data
    }
  })
}