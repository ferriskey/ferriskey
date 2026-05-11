import { useMutation, useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'

export const useSetupOtp = ({ realm, token }: BaseQuery & { token?: string | null }) => {
  return useQuery({
    queryKey: ['setup-otp'],
    queryFn: async (): Promise<Schemas.SetupOtpResponse> => {
      return window.tanstackApi.client.get('/realms/{realm_name}/login-actions/setup-otp', {
        path: { realm_name: realm ?? 'master' },
        header: {
          Authorization: `Bearer ${token}`,
        },
      } as never)
    },
    enabled: !!token,
  })
}

export interface VerifyOtpRequest {
  data: Schemas.OtpVerifyRequest
  token: string
}

export const useVerifyOtp = () => {
  return useMutation({
    mutationFn: async ({ realm, data, token }: BaseQuery & VerifyOtpRequest) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/verify-otp', {
        path: { realm_name: realm ?? 'master' },
        body: data,
        header: {
          Authorization: `Bearer ${token}`,
        },
      } as never) as Promise<Schemas.VerifyOtpResponse>
    },
  })
}

export interface MutationChallengeOtpRequest {
  data: Schemas.ChallengeOtpRequest
  token: string
}

export const useChallengeOtp = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
      token,
    }: BaseQuery & MutationChallengeOtpRequest): Promise<Schemas.ChallengeOtpResponse> => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/challenge-otp', {
        path: { realm_name: realm ?? 'master' },
        body: data,
        header: {
          Authorization: `Bearer ${token}`,
        },
      } as never) as Promise<Schemas.ChallengeOtpResponse>
    },
  })
}

export const useSendMagicLink = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/send-magic-link')
      .mutationOptions,
  })
}

export const useVerifyMagicLink = () => {
  return useMutation({
    ...window.tanstackApi.mutation('get', '/realms/{realm_name}/login-actions/verify-magic-link')
      .mutationOptions,
  })
}

export interface UpdatePasswordRequest {
  data: Schemas.UpdatePasswordRequest
  token: string
}

export const useUpdatePassword = () => {
  return useMutation({
    mutationFn: async ({ realm, data, token }: BaseQuery & UpdatePasswordRequest) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/update-password', {
        path: { realm_name: realm ?? 'master' },
        body: data,
        header: {
          Authorization: `Bearer ${token}`,
        },
      } as never) as Promise<Schemas.UpdatePasswordResponse>
    },
  })
}
