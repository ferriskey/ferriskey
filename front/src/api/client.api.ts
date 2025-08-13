import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery, tanstackApi } from '.'
import { DeleteClientResponse, GetRolesResponse } from './api.interface'
import { Client } from "./core.interface"
import { authStore } from "@/store/auth.store"
import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { UpdateClientSchema } from "@/pages/client/schemas/update-client.schema"
import { toast } from "sonner"

export const useGetClients = ({ realm }: BaseQuery) => {
  return useQuery(
    tanstackApi.get('/realms/{realm_name}/clients', {
      path: {
        realm_name: realm || 'master',
      },
    }).queryOptions
  )
}

export const useGetClient = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
      path: {
        client_id: clientId!,
        realm_name: realm!
      }
    }).queryOptions,
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
        public_client: !payload.clientAuthentication,
        service_account_enabled: !!payload.clientAuthentication,
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

export const useUpdateClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async ({ realm, clientId, payload }: BaseQuery & { clientId: string, payload: UpdateClientSchema }): Promise<Client> => {
      const accessToken = authStore.getState().accessToken

      const response = await apiClient.patch<Client>(`/realms/${realm}/clients/${clientId}`, {
        ...payload,
        client_id: payload.clientId,
      }, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      return response.data
    },
    onSuccess: (client) => {
      toast.success(`Client ${client.name} was updated successfully`)
      queryClient.invalidateQueries({
        queryKey: ["client", client.id],
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
