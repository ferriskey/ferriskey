import { useQuery } from "@tanstack/react-query"
import { apiClient, BaseQuery } from "."
import { SetupOtpResponse } from "./api.interface"

export const useSetupOtp = ({ realm, token }: BaseQuery & { token?: string | null }) => {
  return useQuery({
    queryKey: ['setup-otp'],
    queryFn: async (): Promise<SetupOtpResponse> => {
      const response = await apiClient.get<SetupOtpResponse>(`/realms/${realm}/login-actions/setup-otp`, {
        headers: {
          Authorization: `Bearer ${token}`
        }
      })

      return response.data
    },
    enabled: !!token
  })
}