import { authStore } from "@/store/auth.store"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { CreateUserSchema, UpdateUserSchema } from '../pages/user/validators'
import { CredentialOverview, User } from "./core.interface"
import { BulkDeleteUserResponse, CreateUserResponse, GetUserCredentialsResponse, UpdateUserResponse, UserResponse, UsersResponse, GetUserRolesResponse } from './api.interface'

export interface UserMutateContract<T> {
  realm?: string,
  userId?: string,
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
    queryKey: ["user"],
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

export const useGetUserCredentials = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    queryKey: ["user", "credentials"],
    queryFn: async (): Promise<CredentialOverview[]> => {
      const accessToken = authStore.getState().accessToken

      const { data: responseData } = await apiClient.get<GetUserCredentialsResponse>(`/realms/${realm}/users/${userId}/credentials`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return responseData.data
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

export const useUpdateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, userId, payload }: UserMutateContract<UpdateUserSchema>): Promise<UpdateUserResponse> => {
      const accessToken = authStore.getState().accessToken
      const response = await apiClient.put(`/realms/${realm}/users/${userId}`, payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user"],
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

export const useResetUserPassword = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realm, userId, payload }: UserMutateContract<{ value: string, credential_type: string, temporary: boolean }>): Promise<UpdateUserResponse> => {
      const accessToken = authStore.getState().accessToken
      const response = await apiClient.put(`/realms/${realm}/users/${userId}/reset-password`, payload, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user", "credentials"],
      })
    }
  })
}

export const useGetUserRoles = ({ realm, userId }: BaseQuery & { userId: string }) => {
  return useQuery({
    queryKey: ["user-roles", realm, userId],
    queryFn: async (): Promise<GetUserRolesResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<GetUserRolesResponse>(`/realms/${realm}/users/${userId}/roles`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    enabled: !!realm && !!userId,
  })
}

export const useAssignUserRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realm, userId, payload }: UserMutateContract<{ roleId: string }>) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.post(`/realms/${realm}/users/${userId}/roles/${payload.roleId}`, {}, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["user-roles"]
      })
    }
  })
}