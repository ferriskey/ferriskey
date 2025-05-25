import { authStore } from "@/store/auth.store"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { CreateUserSchema } from '../pages/user/validators'
import { BulkDeleteUserResponse, CreateUserResponse, User, UserResponse, UsersResponse } from "./api.interface"

export interface UserMutateContract<T> {
  realm: string,
  payload: T
}

export interface GetUserQueryParams {
  realm?: string
  userId?: string
}

export const useGetUsers = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["users"],
    queryFn: async (): Promise<User[]> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<UsersResponse>(`/realms/${realm}/users`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data.data
    }
  })
}

export const useGetUser = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    queryKey: ["user", userId],
    queryFn: async (): Promise<User> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<UserResponse>(`/realms/${realm}/users/${userId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data.data
    },
    enabled: !!userId && !!realm
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