import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

export const useGetSmtpConfig = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/smtp-config', {
      path: { realm_name: realm! },
    }).queryOptions,
    enabled: !!realm,
    retry: false,
  })
}

export const useUpsertSmtpConfig = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/smtp-config').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/smtp-config', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.smtp.saved'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.smtp.save_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteSmtpConfig = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/smtp-config').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/smtp-config', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.smtp.deleted'))
    },
    onError: (error) => {
      toast.error(translate('common:toast.smtp.delete_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}
