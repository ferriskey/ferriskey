import { useMutation, useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'

export const useSetupOtp = ({ realm, enabled = true }: BaseQuery & { enabled?: boolean }) => {
  return useQuery({
    queryKey: ['setup-otp', realm ?? 'master'],
    queryFn: async (): Promise<Schemas.SetupOtpResponse> => {
      return window.tanstackApi.client.get('/realms/{realm_name}/login-actions/setup-otp', {
        path: { realm_name: realm ?? 'master' },
      } as never)
    },
    enabled,
  })
}

export interface VerifyOtpRequest {
  data: Schemas.OtpVerifyRequest
}

export const useVerifyOtp = () => {
  return useMutation({
    mutationFn: async ({ realm, data }: BaseQuery & VerifyOtpRequest) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/verify-otp', {
        path: { realm_name: realm ?? 'master' },
        body: data,
      } as never) as Promise<Schemas.VerifyOtpResponse>
    },
  })
}

export interface MutationChallengeOtpRequest {
  data: Schemas.ChallengeOtpRequest
}

export const useChallengeOtp = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & MutationChallengeOtpRequest): Promise<Schemas.ChallengeOtpResponse> => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/challenge-otp', {
        path: { realm_name: realm ?? 'master' },
        body: data,
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
}

export const useUpdatePassword = () => {
  return useMutation({
    mutationFn: async ({ realm, data }: BaseQuery & UpdatePasswordRequest) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/update-password', {
        path: { realm_name: realm ?? 'master' },
        body: data,
      } as never) as Promise<Schemas.UpdatePasswordResponse>
    },
  })
}
