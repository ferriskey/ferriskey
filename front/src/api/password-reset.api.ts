import { useMutation } from '@tanstack/react-query'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

export const useForgotPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/forgot-password')
      .mutationOptions,
    onSuccess: () => {
      toast.success(translate('common:toast.password_reset.link_sent'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.password_reset.link_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useResetPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/reset-password')
      .mutationOptions,
    onError: (error) => {
      toast.error(translate('common:toast.password_reset.reset_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useVerifyResetToken = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/login-actions/verify-reset-token')
      .mutationOptions,
  })
}
