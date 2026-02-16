import { useMutation, useQuery } from '@tanstack/react-query'
import type { PostEndpoints, Schemas } from './api.client'

const REVOKE_TOKEN_PATH =
  '/realms/{realm_name}/protocol/openid-connect/revoke' as unknown as keyof PostEndpoints

type AuthQueryParams = {
  response_type?: string
  client_id?: string
  redirect_uri?: string
  scope?: string
  state?: string
}

export interface AuthenticatePayload {
  data: Schemas.AuthenticateRequest
  realm: string
  clientId: string
  sessionCode: string
  useToken?: boolean
  token?: string
}

export interface AuthQuery {
  query: string
  realm: string
}

const parseAuthQuery = (query: string): AuthQueryParams => {
  const params = new URLSearchParams(query)

  return {
    response_type: params.get('response_type') ?? undefined,
    client_id: params.get('client_id') ?? undefined,
    redirect_uri: params.get('redirect_uri') ?? undefined,
    scope: params.get('scope') ?? undefined,
    state: params.get('state') ?? undefined,
  }
}

export const useAuthQuery = (params: AuthQuery) => {
  const query = parseAuthQuery(params.query)

  return useQuery<Schemas.AuthResponse>({
    queryKey: ['auth', params.realm, params.query],
    queryFn: async (): Promise<Schemas.AuthResponse> =>
      (await window.tanstackApi.client.get('/realms/{realm_name}/protocol/openid-connect/auth', {
        path: { realm_name: params.realm },
        query,
      })) as Schemas.AuthResponse,
  })
}

export const useAuthenticateMutation = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/authenticate')
      .mutationOptions,
    mutationFn: async (params: AuthenticatePayload): Promise<Schemas.AuthenticateResponse> => {
      const headers: Record<string, string> = {}

      if (params.token !== undefined) {
        headers.Authorization = `Bearer ${params.token}`
      }

      return window.tanstackApi.client.post('/realms/{realm_name}/login-actions/authenticate', {
        path: { realm_name: params.realm },
        query: {
          client_id: params.clientId,
          session_code: params.sessionCode,
        },
        body: params.data,
        ...(Object.keys(headers).length > 0 ? { header: headers } : {}),
      } as never)
    },
  })
}

export interface TokenPayload {
  data: Schemas.TokenRequestValidator
  realm: string
}

export const useTokenMutation = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/protocol/openid-connect/token')
      .mutationOptions,
    mutationFn: async (params: TokenPayload): Promise<Schemas.JwtToken> => {
      const formData = new URLSearchParams()

      formData.append('grant_type', params.data.grant_type!)
      formData.append('client_id', params.data.client_id!)

      if (params.data.client_secret) {
        formData.append('client_secret', params.data.client_secret)
      }
      if (params.data.code) {
        formData.append('code', params.data.code)
      }
      if (params.data.username) {
        formData.append('username', params.data.username)
      }
      if (params.data.password) {
        formData.append('password', params.data.password)
      }
      if (params.data.refresh_token) {
        formData.append('refresh_token', params.data.refresh_token)
      }

      return (await window.tanstackApi.client.post(
        '/realms/{realm_name}/protocol/openid-connect/token',
        {
          path: { realm_name: params.realm },
          body: formData,
          header: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
        } as never
      )) as Schemas.JwtToken
    },
  })
}

export const useRegistrationMutation = () => {
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/protocol/openid-connect/registrations',
      async (res) => res.json()
    ).mutationOptions,
  })
}

export interface RevokeTokenPayload {
  realm: string
  data: {
    token: string
    client_id: string
    token_type_hint?: string
  }
}

export const useRevokeTokenMutation = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', REVOKE_TOKEN_PATH).mutationOptions,
    mutationFn: async (params: RevokeTokenPayload): Promise<void> => {
      const formData = new URLSearchParams()
      formData.append('token', params.data.token)
      formData.append('client_id', params.data.client_id)

      if (params.data.token_type_hint) {
        formData.append('token_type_hint', params.data.token_type_hint)
      }

      await window.tanstackApi.client.post(REVOKE_TOKEN_PATH, {
        path: { realm_name: params.realm },
        body: formData,
        header: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      } as never)
    },
  })
}

export interface LogoutPayload {
  realm: string
  data?: {
    id_token_hint?: string
    post_logout_redirect_uri?: string
    state?: string
    client_id?: string
  }
}

export const useLogoutMutation = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/protocol/openid-connect/logout')
      .mutationOptions,
    mutationFn: async (params: LogoutPayload): Promise<void> => {
      const formData = new URLSearchParams()

      if (params.data?.id_token_hint) {
        formData.append('id_token_hint', params.data.id_token_hint)
      }
      if (params.data?.post_logout_redirect_uri) {
        formData.append('post_logout_redirect_uri', params.data.post_logout_redirect_uri)
      }
      if (params.data?.state) {
        formData.append('state', params.data.state)
      }
      if (params.data?.client_id) {
        formData.append('client_id', params.data.client_id)
      }

      const hasPayload = formData.toString().length > 0

      await window.tanstackApi.client.post('/realms/{realm_name}/protocol/openid-connect/logout', {
        path: { realm_name: params.realm },
        ...(hasPayload
          ? {
              body: formData,
              header: {
                'Content-Type': 'application/x-www-form-urlencoded',
              },
            }
          : {}),
      } as never)
    },
  })
}
