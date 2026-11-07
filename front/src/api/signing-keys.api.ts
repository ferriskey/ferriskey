import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

export type SigningKey = Schemas.SigningKey
export type SigningKeyStatus = Schemas.SigningKeyStatus

const signingKeysKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/keys', {
    path: { realm_name: realm },
  }).queryKey

export const useGetSigningKeys = ({ realm }: { realm?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/keys', {
      path: { realm_name: realm ?? '' },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useRotateSigningKey = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ realm }: { realm: string }) =>
      window.api.post('/realms/{realm_name}/keys/rotate', {
        path: { realm_name: realm },
      }),
    onSuccess: async (_response, variables) => {
      await queryClient.invalidateQueries({ queryKey: signingKeysKey(variables.realm) })
    },
  })
}

export const useRevokeSigningKey = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ realm, kid }: { realm: string; kid: string }) =>
      window.api.post('/realms/{realm_name}/keys/{kid}/revoke', {
        path: { realm_name: realm, kid },
      }),
    onSuccess: async (_response, variables) => {
      await queryClient.invalidateQueries({ queryKey: signingKeysKey(variables.realm) })
    },
  })
}
