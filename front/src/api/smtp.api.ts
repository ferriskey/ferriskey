import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

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
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/smtp-config', async (res) => res.json())
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/smtp-config', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('SMTP configuration saved successfully')
    },
    onError: (error: Error) => {
      toast.error('Failed to save SMTP configuration', { description: error.message })
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
      toast.success('SMTP configuration deleted')
    },
    onError: (error: Error) => {
      toast.error('Failed to delete SMTP configuration', { description: error.message })
    },
  })
}
