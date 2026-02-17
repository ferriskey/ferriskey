import { useMutation, useQuery } from '@tanstack/react-query'
import type { PostEndpoints, Schemas } from './api.client'

const TOKEN_PATH: keyof PostEndpoints = '/realms/{realm_name}/protocol/openid-connect/token'
const LOGOUT_PATH: keyof PostEndpoints = '/realms/{realm_name}/protocol/openid-connect/logout'
const REVOKE_TOKEN_PATH: keyof PostEndpoints = '/realms/{realm_name}/protocol/openid-connect/revoke'

type PostParameters<Path extends keyof PostEndpoints> =
  PostEndpoints[Path] extends { parameters: infer Parameters } ? Parameters : never

type PostResponse<Path extends keyof PostEndpoints> =
  PostEndpoints[Path] extends { response: infer Response } ? Response : never

const postUrlEncoded = async <Path extends keyof PostEndpoints>(
  path: Path,
  params: Omit<PostParameters<Path>, 'body'> & { body: Record<string, unknown> }
): Promise<PostResponse<Path>> => {
  const formData = new URLSearchParams()

  Object.entries(params.body).forEach(([key, value]) => {
    if (value !== undefined && value !== null) {
      formData.append(key, String(value))
    }
  })

  return window.tanstackApi.client.post(path, {
    ...params,
    body: formData,
    header: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
  } as never) as Promise<PostResponse<Path>>
}

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

  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/protocol/openid-connect/auth', {
      path: { realm_name: params.realm },
      query,
    }).queryOptions
  )
}

export const useAuthenticateMutation = () => {
  const authenticateMutation = window.tanstackApi.mutation(
    'post',
    '/realms/{realm_name}/login-actions/authenticate',
    async (res) => res.json()
  )

  return useMutation({
    ...authenticateMutation.mutationOptions,
    mutationFn: async (params: AuthenticatePayload): Promise<Schemas.AuthenticateResponse> => {
      const headers: Record<string, string> = {}

      if (params.token !== undefined) {
        headers.Authorization = `Bearer ${params.token}`
      }

      return authenticateMutation.mutationOptions.mutationFn({
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
    ...window.tanstackApi.mutation('post', TOKEN_PATH).mutationOptions,
    mutationFn: async (params: TokenPayload): Promise<Schemas.JwtToken> => {
      if (!params.data.grant_type || !params.data.client_id) {
        throw new Error('grant_type and client_id are required')
      }

      return (await postUrlEncoded(TOKEN_PATH, {
        path: { realm_name: params.realm },
        body: {
          grant_type: params.data.grant_type,
          client_id: params.data.client_id,
          client_secret: params.data.client_secret,
          code: params.data.code,
          username: params.data.username,
          password: params.data.password,
          refresh_token: params.data.refresh_token,
          scope: params.data.scope,
        },
      })) as Schemas.JwtToken
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
      await postUrlEncoded(REVOKE_TOKEN_PATH, {
        path: { realm_name: params.realm },
        body: {
          token: params.data.token,
          client_id: params.data.client_id,
          token_type_hint: params.data.token_type_hint,
        },
      })
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
    ...window.tanstackApi.mutation('post', LOGOUT_PATH).mutationOptions,
    mutationFn: async (params: LogoutPayload): Promise<void> => {
      await postUrlEncoded(LOGOUT_PATH, {
        path: { realm_name: params.realm },
        body: {
          id_token_hint: params.data?.id_token_hint,
          post_logout_redirect_uri: params.data?.post_logout_redirect_uri,
          state: params.data?.state,
          client_id: params.data?.client_id,
        },
      })
    },
  })
}
