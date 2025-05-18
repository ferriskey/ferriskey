import { useQuery } from "@tanstack/react-query"
import { apiClient } from "."
import { userStore } from "@/store/user.store"
import { Realm, UserRealmsResponse } from "./api.interface"

export interface UserRealmsQuery {
  realm: string
}

export const useGetUserRealmsQuery = ({ realm }: UserRealmsQuery) => {
  return useQuery({
    queryKey: ["user-realms"],
    queryFn: async (): Promise<Realm[]> => {
      const access_token = userStore.getState().access_token
      const response = await apiClient.get<UserRealmsResponse>(`/realms/${realm}/users/@me/realms`, {
        headers: {
          Authorization: `Bearer ${access_token}`,
        },
      })

      return response.data.data
    }
  })
}