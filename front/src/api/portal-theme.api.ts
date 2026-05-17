import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetPortalTheme = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal/theme').mutationOptions,
    onSuccess: async (_, variables) => {
      const themeKeys = window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      const loginSettingsKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: { name: variables.path.realm_name },
      }).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: themeKeys }),
        queryClient.invalidateQueries({ queryKey: loginSettingsKeys }),
      ])

      toast.success('Portal theme saved')
    },
  })
}
