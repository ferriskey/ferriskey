import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { Client, ClientsResponse, DeleteClientResponse, GetClientResponse, GetRolesResponse } from './api.interface'
import { authStore } from "@/store/auth.store"
import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'


export const useGetClients = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: ["clients"],
    queryFn: async (): Promise<ClientsResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<ClientsResponse>(`/realms/${realm}/clients`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    }
  })
}

export const useGetClient = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    queryKey: ["client", clientId],
    queryFn: async (): Promise<Client> => {
      const accessToken = authStore.getState().accessToken

      const { data: response } = await apiClient.get<GetClientResponse>(`/realms/${realm}/clients/${clientId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })
      return response.data
    },
    enabled: !!clientId && !!realm,
  })
}

export interface CreateClientMutate {
  realm: string,
  payload: CreateClientSchema
}

export const useCreateClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, payload }: CreateClientMutate) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.post(`/realms/${realm}/clients`, {
        ...payload,
        client_id: payload.clientId,
        client_type: payload.clientAuthentication ? "confidential" : "public",
        public_client: payload.clientAuthentication ? false : true,
        service_account_enabled: payload.clientAuthentication ? true : false,
      }, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["clients"],
      })
    }
  })
}

export const useDeleteClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, clientId }: BaseQuery & { clientId: string }) => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.delete<DeleteClientResponse>(`/realms/${realm}/clients/${clientId}`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["clients"],
      })
    }
  })
}

export const useGetClientRoles = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    queryKey: ["client-roles", realm, clientId],
    queryFn: async (): Promise<GetRolesResponse> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.get<GetRolesResponse>(`/realms/${realm}/clients/${clientId}/roles`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    enabled: !!clientId && !!realm,
  })
}
