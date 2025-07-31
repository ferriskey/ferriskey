import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { apiClient } from "."
import { Realm } from "./core.interface"
import { UserRealmsResponse } from './api.interface'
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

export const useCreateRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ payload }: { payload: { name: string }}) => {
      const accessToken = authStore.getState().accessToken

      console.log("Creating realm with payload:", payload)

      const response = await apiClient.post('/realms', payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user-realms"]
      })
    }
  })
}
