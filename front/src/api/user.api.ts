import { userStore } from "@/store/user.store"
import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { UsersResponse } from "./api.interface"


export const useGetUsers = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["users"],
    queryFn: async (): Promise<UsersResponse> => {
      const access_token = userStore.getState().access_token

      const response = await apiClient.get<UsersResponse>(`/realms/${realm}/users`, {
        headers: {
          Authorization: `Bearer ${access_token}`,
        },
      })

      return response.data
    }
  })
}