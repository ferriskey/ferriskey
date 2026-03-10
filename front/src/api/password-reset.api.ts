import { useMutation } from '@tanstack/react-query'
import { toast } from 'sonner'

export const useForgotPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/forgot-password')
      .mutationOptions,
    onSuccess: () => {
      toast.success('If an account exists with this email, a reset link has been sent')
    },
    onError: (error) => {
      toast.error('Failed to send reset link', { description: error.message })
    },
  })
}

export const useResetPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/reset-password')
      .mutationOptions,
    onSuccess: () => {
      toast.success('Your password has been reset successfully')
    },
    onError: (error) => {
      toast.error('Failed to reset password', { description: error.message })
    },
  })
}
