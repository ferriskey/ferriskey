import { useQuery } from "@tanstack/react-query"
import { apiClient } from "."
import { Realm, UserRealmsResponse } from "./api.interface"
import { authStore } from "@/store/auth.store"

export interface UserRealmsQuery {
  realm: string
}

export const useGetUserRealmsQuery = ({ realm }: UserRealmsQuery) => {
  return useQuery({
    queryKey: ["user-realms"],
    queryFn: async (): Promise<Realm[]> => {
      const accessToken = authStore.getState().accessToken
      const response = await apiClient.get<UserRealmsResponse>(`/realms/${realm}/users/@me/realms`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data.data
    }
  })
}