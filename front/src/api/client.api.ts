import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { ClientsResponse } from "./api.interface"
import { CreateClientSchema } from "@/pages/client/feature/create-client-modal-feature"
import { authStore } from "@/store/auth.store"


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