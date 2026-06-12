import { useMutation } from '@tanstack/react-query'
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
