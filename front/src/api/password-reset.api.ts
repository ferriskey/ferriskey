import { useMutation } from '@tanstack/react-query'

export interface ForgotPasswordPayload {
  realm: string
  email: string
}

export const useForgotPasswordMutation = () => {
  return useMutation({
    mutationFn: async (params: ForgotPasswordPayload): Promise<void> => {
      const response = await fetch(
        `${window.apiUrl}/realms/${params.realm}/login-actions/forgot-password`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email: params.email }),
        }
      )

      if (!response.ok && response.status !== 204) {
        let errorMessage = 'An error occurred'
        try {
          const errorBody = await response.json()
          if (errorBody.message) errorMessage = errorBody.message
        } catch {
          // keep default
        }
        throw new Error(errorMessage)
      }
    },
  })
}

export interface ResetPasswordPayload {
  realm: string
  token_id: string
  token: string
  new_password: string
}

export const useResetPasswordMutation = () => {
  return useMutation({
    mutationFn: async (params: ResetPasswordPayload): Promise<void> => {
      const response = await fetch(
        `${window.apiUrl}/realms/${params.realm}/login-actions/reset-password`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            token_id: params.token_id,
            token: params.token,
            new_password: params.new_password,
          }),
        }
      )

      if (!response.ok) {
        let errorMessage = 'An error occurred'
        try {
          const errorBody = await response.json()
          if (errorBody.message) errorMessage = errorBody.message
        } catch {
          // keep default
        }
        throw new Error(errorMessage)
      }
    },
  })
}
