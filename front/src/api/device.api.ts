import { useMutation, useQuery } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'

export const useDeviceVerify = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & { data: Schemas.DeviceVerifyRequest }) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/device/verify', {
        path: { realm_name: realm ?? 'master' },
        body: data,
      } as never) as Promise<Schemas.DeviceVerifyResponse>
    },
  })
}

export const useDevicePreview = ({
  realm,
  userCode,
  enabled,
}: BaseQuery & { userCode: string; enabled: boolean }) => {
  return useQuery({
    queryKey: ['device-preview', realm, userCode],
    enabled,
    retry: false,
    staleTime: 0,
    queryFn: async () => {
      return window.tanstackApi.client.get('/realms/{realm_name}/device/preview', {
        path: { realm_name: realm ?? 'master' },
        query: { user_code: userCode },
      } as never) as Promise<Schemas.DeviceVerificationPreview>
    },
  })
}
