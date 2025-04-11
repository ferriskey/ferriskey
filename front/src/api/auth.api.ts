import { useMutation, useQuery } from '@tanstack/react-query'
import { apiClient } from '.'
import { AuthenticateRequest, AuthResponse } from './api.interface'

export interface AuthenticatePayload {
  data: AuthenticateRequest
  realm: string
  clientId: string
  sessionCode: string
}

export interface AuthQuery {
  query: string
  realm: string
}

const BASE_URL = 'http://localhost:3333' // Adjust this based on your API base URL

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
    mutationFn: async (params: AuthenticatePayload) => {
      const response = await apiClient.post(
        `/realms/${params.realm}/login-actions/authenticate?client_id=${params.clientId}&session_code=${params.sessionCode}`,
        params.data
      )

      return response.data
    },
  })
}
