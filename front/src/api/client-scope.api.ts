import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import { toast } from 'sonner'

export const useGetClientScopes = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useGetClientScope = ({ realm, scopeId }: BaseQuery & { scopeId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/client-scopes/{scope_id}', {
      path: {
        realm_name: realm!,
        scope_id: scopeId!,
      },
    }).queryOptions,
    enabled: !!realm && !!scopeId,
  })
}

export const useCreateClientScope = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/client-scopes', async (res) =>
      res.json()
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const { queryKey } = window.tanstackApi.get('/realms/{realm_name}/client-scopes', {
        path: {
          realm_name: variables.path.realm_name,
        },
      })
      await queryClient.invalidateQueries({ queryKey })
      toast.success('Client scope created successfully')
    },
    onError: (error) => {
      toast.error('Failed to create client scope', {
        description: error.message,
      })
    },
  })
}
