import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export function useGetClientWhitelist({
  realm = 'master',
  clientId,
}: BaseQuery & { clientId?: string }) {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}/maintenance/whitelist', {
      path: { realm_name: realm, client_id: clientId! },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export function useToggleMaintenance() {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/clients/{client_id}/maintenance')
      .mutationOptions,
    onSuccess: async (data, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
        path: {
          client_id: variables.path.client_id,
          realm_name: variables.path.realm_name,
        },
      }).queryKey

      toast.success(data.message)
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export function useAddClientWhitelistEntry() {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name, client_id: variables.path.client_id } }
      ).queryKey

      toast.success('Whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export function useRemoveClientWhitelistEntry() {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name, client_id: variables.path.client_id } }
      ).queryKey

      toast.success('Whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

// -- Realm maintenance whitelist --

export function useGetRealmWhitelist({ realm = 'master' }: BaseQuery) {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/clients/settings/maintenance/whitelist', {
      path: { realm_name: realm },
    }).queryOptions
  )
}

export function useAddRealmWhitelistEntry() {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/clients/settings/maintenance/whitelist'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/clients/settings/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name } }
      ).queryKey

      toast.success('Realm whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export function useRemoveRealmWhitelistEntry() {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/clients/settings/maintenance/whitelist/{entry_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/clients/settings/maintenance/whitelist',
        { path: { realm_name: variables.path.realm_name } }
      ).queryKey

      toast.success('Realm whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
