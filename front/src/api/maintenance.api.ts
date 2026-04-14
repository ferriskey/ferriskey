import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

// Types for maintenance endpoints (not yet in the auto-generated OpenAPI client).

export interface WhitelistEntry {
  id: string
  client_id?: string
  realm_id?: string
  user_id?: string
  role_id?: string
  created_at: string
}

interface ToggleMaintenanceResponse {
  message: string
}

interface WhitelistResponse {
  data: WhitelistEntry[]
}

// Cast helpers — the tanstackApi methods are strongly typed to known paths only.
// These endpoints exist at runtime but aren't yet in the generated type map.
// We use a generic QueryResult so useQuery can infer the return type.

interface PathVars {
  path: Record<string, string>
}

interface QueryResult<T> {
  queryOptions: { queryKey: unknown[]; queryFn: () => Promise<T> }
  queryKey: unknown[]
}

interface MutationResult {
  mutationOptions: object
}

const apiGet = window.tanstackApi.get as unknown as <T>(
  path: string,
  vars: PathVars
) => QueryResult<T>

const apiMutation = window.tanstackApi.mutation as unknown as (
  method: string,
  path: string
) => MutationResult

interface MutationVars {
  path: Record<string, string>
}

// -- Client maintenance whitelist --

export const useGetClientWhitelist = ({
  realm = 'master',
  clientId,
}: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...apiGet<WhitelistResponse>('/realms/{realm_name}/clients/{client_id}/maintenance/whitelist', {
      path: { realm_name: realm, client_id: clientId! },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export const useToggleMaintenance = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...apiMutation('put', '/realms/{realm_name}/clients/{client_id}/maintenance').mutationOptions,
    onSuccess: async (data: unknown, variables: unknown) => {
      const vars = variables as MutationVars
      const resp = data as ToggleMaintenanceResponse
      const keys = window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
        path: {
          client_id: vars.path.client_id,
          realm_name: vars.path.realm_name,
        },
      }).queryKey
      toast.success(resp.message)
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useAddClientWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...apiMutation('post', '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist')
      .mutationOptions,
    onSuccess: async (_data: unknown, variables: unknown) => {
      const vars = variables as MutationVars
      const keys = apiGet<WhitelistResponse>(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: vars.path.realm_name, client_id: vars.path.client_id } }
      ).queryKey
      toast.success('Whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useRemoveClientWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...apiMutation(
      'delete',
      '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}'
    ).mutationOptions,
    onSuccess: async (_data: unknown, variables: unknown) => {
      const vars = variables as MutationVars
      const keys = apiGet<WhitelistResponse>(
        '/realms/{realm_name}/clients/{client_id}/maintenance/whitelist',
        { path: { realm_name: vars.path.realm_name, client_id: vars.path.client_id } }
      ).queryKey
      toast.success('Whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

// -- Realm maintenance whitelist --

export const useGetRealmWhitelist = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    apiGet<WhitelistResponse>('/realms/{realm_name}/settings/maintenance/whitelist', {
      path: { realm_name: realm },
    }).queryOptions
  )
}

export const useAddRealmWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...apiMutation('post', '/realms/{realm_name}/settings/maintenance/whitelist').mutationOptions,
    onSuccess: async (_data: unknown, variables: unknown) => {
      const vars = variables as MutationVars
      const keys = apiGet<WhitelistResponse>(
        '/realms/{realm_name}/settings/maintenance/whitelist',
        { path: { realm_name: vars.path.realm_name } }
      ).queryKey
      toast.success('Realm whitelist entry added')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useRemoveRealmWhitelistEntry = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...apiMutation('delete', '/realms/{realm_name}/settings/maintenance/whitelist/{entry_id}')
      .mutationOptions,
    onSuccess: async (_data: unknown, variables: unknown) => {
      const vars = variables as MutationVars
      const keys = apiGet<WhitelistResponse>(
        '/realms/{realm_name}/settings/maintenance/whitelist',
        { path: { realm_name: vars.path.realm_name } }
      ).queryKey
      toast.success('Realm whitelist entry removed')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
