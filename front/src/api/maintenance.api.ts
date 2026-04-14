import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

// -- Client maintenance whitelist --

export const useGetClientWhitelist = ({
  realm = 'master',
  clientId,
}: BaseQuery & { clientId?: string }) => {
  return useQuery({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.get as any)(
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
      { path: { realm_name: realm, client_id: clientId! } }
    ).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export const useToggleMaintenance = () => {
  const queryClient = useQueryClient()
  return useMutation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.mutation as any)(
      'put',
      '/realms/{realm_name}/clients/{client_id}/maintenance'
    ).mutationOptions,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onSuccess: async (data: any, variables: any) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/clients/{client_id}',
        {
          path: {
            client_id: variables.path.client_id,
            realm_name: variables.path.realm_name,
          },
        }
      ).queryKey
      toast.success(data.message)
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useAddClientWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.mutation as any)(
      'post',
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist'
    ).mutationOptions,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onSuccess: async (_: any, variables: any) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const keys = (window.tanstackApi.get as any)(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name, client_id: variables.path.client_id } }
      ).queryKey
      toast.success('Whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useRemoveClientWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.mutation as any)(
      'delete',
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}'
    ).mutationOptions,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onSuccess: async (_: any, variables: any) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const keys = (window.tanstackApi.get as any)(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name, client_id: variables.path.client_id } }
      ).queryKey
      toast.success('Whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

// -- Realm maintenance whitelist --

export const useGetRealmWhitelist = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window.tanstackApi.get as any)(
      '/realms/{realm_name}/settings/maintenance/whitelist',
      { path: { realm_name: realm } }
    ).queryOptions
  )
}

export const useAddRealmWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.mutation as any)(
      'post',
      '/realms/{realm_name}/settings/maintenance/whitelist'
    ).mutationOptions,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onSuccess: async (_: any, variables: any) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const keys = (window.tanstackApi.get as any)(
        '/realms/{realm_name}/settings/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name } }
      ).queryKey
      toast.success('Realm whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useRemoveRealmWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ...(window.tanstackApi.mutation as any)(
      'delete',
      '/realms/{realm_name}/settings/maintenance/whitelist/{entry_id}'
    ).mutationOptions,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onSuccess: async (_: any, variables: any) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const keys = (window.tanstackApi.get as any)(
        '/realms/{realm_name}/settings/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name } }
      ).queryKey
      toast.success('Realm whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
