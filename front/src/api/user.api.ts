import { userStore } from "@/store/user.store"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { CreateUserSchema } from '../pages/user/validators'
import { BulkDeleteUserResponse, CreateUserResponse, UsersResponse } from "./api.interface"

export interface UserMutateContract<T> {
  realm: string,
  payload: T
}

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

export const useCreateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, payload }: UserMutateContract<CreateUserSchema>): Promise<CreateUserResponse> => {
      const accessToken = userStore.getState().access_token
      const response = await apiClient.post(`/realms/${realm}/users`, payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["users"],
      })
    }
  })
}

export const useBulkDeleteUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, payload }: UserMutateContract<{ ids: string[] }>): Promise<BulkDeleteUserResponse> => {
      const accessToken = userStore.getState().access_token
      const response = await apiClient.delete(`/realms/${realm}/users/bulk`, {
        data: payload,
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["users"],
      })
    }
  })
}