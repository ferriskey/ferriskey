import { useMutation, useQuery } from '@tanstack/react-query'
import { apiClient } from '.'
import {
  AuthenticateRequest,
  AuthenticateResponse,
  AuthResponse,
  JwtToken,
  TokenRequestValidator,
} from './api.interface'

export interface AuthenticatePayload {
  data: AuthenticateRequest
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

export const useAuthQuery = (params: AuthQuery) => {
  return useQuery({
    queryKey: ['auth'],
    queryFn: async (): Promise<AuthResponse> => {
      const response = await apiClient.get<AuthResponse>(
        `/realms/${params.realm}/protocol/openid-connect/auth?${params.query}`
      )

      return response.data
    },
  })
}

export const useAuthenticateMutation = () => {
  return useMutation({
    mutationFn: async (params: AuthenticatePayload): Promise<AuthenticateResponse> => {
      const headers: Record<string, string> = {}

      if (params.token !== undefined) {
        headers.Authorization = `Bearer ${params.token}`
      }

      const response = await apiClient.post<AuthenticateResponse>(
        `/realms/${params.realm}/login-actions/authenticate?client_id=${params.clientId}&session_code=${params.sessionCode}`,
        params.data,
        {
          headers,
        }
      )

      return response.data
    },
  })
}

export interface TokenPayload {
  data: TokenRequestValidator
  realm: string
}

export const useTokenMutation = () => {
  return useMutation({
    mutationFn: async (params: TokenPayload): Promise<JwtToken> => {
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

      const response = await apiClient.post<JwtToken>(
        `/realms/${params.realm}/protocol/openid-connect/token`,
        formData,
        {
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          withCredentials: true,
        }
      )

      return response.data
    },
  })
}
