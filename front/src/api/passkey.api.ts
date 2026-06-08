import { useMutation } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'

export const usePasskeyRequestOptionsMutation = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & { data: Schemas.PasskeyRequestOptionsRequest }) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/passkey-request-options', {
        path: { realm_name: realm ?? 'master' },
        body: data,
      } as never) as Promise<Schemas.PasskeyPublicKeyCredentialRequestOptionsJSON>
    },
  })
}

export const usePasskeyAuthenticateMutation = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & { data: Schemas.PasskeyPublicKeyCredential }) => {
      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/passkey-authenticate', {
        path: { realm_name: realm ?? 'master' },
        body: data,
      } as never) as Promise<Schemas.PasskeyAuthenticateResponse>
    },
  })
}
