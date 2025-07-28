import { useMutation, useQuery } from '@tanstack/react-query'
import { apiClient, BaseQuery } from '.'
import {
  ChallengeOtpRequest,
  ChallengeOtpResponse,
  OtpVerifyRequest,
  SetupOtpResponse,
  VerifyOtpResponse,
} from './api.interface'

export const useSetupOtp = ({ realm, token }: BaseQuery & { token?: string | null }) => {
  return useQuery({
    queryKey: ['setup-otp'],
    queryFn: async (): Promise<SetupOtpResponse> => {
      const response = await apiClient.get<SetupOtpResponse>(
        `/realms/${realm}/login-actions/setup-otp`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      )

      return response.data
    },
    enabled: !!token,
  })
}

export interface VerifyOtpRequest {
  data: OtpVerifyRequest
  token: string
}

export const useVerifyOtp = () => {
  return useMutation({
    mutationFn: async ({ realm, data, token }: BaseQuery & VerifyOtpRequest) => {
      const response = await apiClient.post<VerifyOtpResponse>(
        `/realms/${realm}/login-actions/verify-otp`,
        data,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      )

      return response.data
    },
  })
}

export interface MutationChallengeOtpRequest {
  data: ChallengeOtpRequest
  token: string
}

export const useChallengeOtp = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
      token,
    }: BaseQuery & MutationChallengeOtpRequest): Promise<ChallengeOtpResponse> => {
      const response = await apiClient.post<ChallengeOtpResponse>(
        `/realms/${realm}/login-actions/challenge-otp`,
        data,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      )

      return response.data
    },
  })
}
