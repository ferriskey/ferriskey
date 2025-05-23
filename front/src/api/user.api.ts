import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { CreateUserSchema } from '../pages/user/validators'
import { BulkDeleteUserResponse, CreateUserResponse, UsersResponse } from "./api.interface"
import { authStore } from "@/store/auth.store"

export interface UserMutateContract<T> {
  realm: string,
  payload: T
}

export const useGetUsers = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["users"],
    queryFn: async (): Promise<UsersResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<UsersResponse>(`/realms/${realm}/users`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
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
      const accessToken = authStore.getState().accessToken
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
      const accessToken = authStore.getState().accessToken
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