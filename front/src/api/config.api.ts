import { useQuery } from "@tanstack/react-query"
import { apiClient } from "."
import { GetConfigResponse } from "./api.interface"

export const useGetConfig = () => {
  return useQuery({
    queryKey: ["config"],
    queryFn: async (): Promise<GetConfigResponse> => {
      const response = await apiClient.get<GetConfigResponse>("/config")

      return response.data
    }
  })
}