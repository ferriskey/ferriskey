import { useMutation } from '@tanstack/react-query'
import { BaseQuery } from '.'

export interface PasskeyRequestOptionsRequest {
  username?: string
}

export interface PasskeyPublicKeyOptions {
  challenge: string
  timeout?: number
  rpId?: string
  allowCredentials?: Array<{
    type: string
    id: string
    transports?: string[]
  }>
  userVerification?: string
  extensions?: Record<string, unknown>
}

export interface PasskeyRequestOptionsResponse {
  publicKey: PasskeyPublicKeyOptions
  mediation?: string
}

export interface PasskeyAuthenticateResponse {
  login_url: string
  status: string
}

export const usePasskeyRequestOptionsMutation = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & { data: PasskeyRequestOptionsRequest }): Promise<PasskeyRequestOptionsResponse> => {
      const response = await window.axios.post<PasskeyRequestOptionsResponse>(
        `/realms/${realm}/login-actions/passkey-request-options`,
        data
      )
      return response.data
    },
  })
}

export const usePasskeyAuthenticateMutation = () => {
  return useMutation({
    mutationFn: async ({
      realm,
      data,
    }: BaseQuery & { data: Record<string, unknown> }): Promise<PasskeyAuthenticateResponse> => {
      const response = await window.axios.post<PasskeyAuthenticateResponse>(
        `/realms/${realm}/login-actions/passkey-authenticate`,
        data
      )
      return response.data
    },
  })
}
