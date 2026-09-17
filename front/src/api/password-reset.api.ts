import { useMutation } from '@tanstack/react-query'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'

export const useForgotPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/forgot-password')
      .mutationOptions,
    onSuccess: () => {
      toast.success('If an account exists with this email, a reset link has been sent')
    },
    onError: (error) => {
      toast.error('Failed to send reset link', { description: apiErrorMessage(error) })
    },
  })
}

export const useResetPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/reset-password')
      .mutationOptions,
    onError: (error) => {
      toast.error('Failed to reset password', { description: apiErrorMessage(error) })
    },
  })
}

export const useVerifyResetToken = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/verify-reset-token')
      .mutationOptions,
  })
}
