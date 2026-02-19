import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetClients = ({ realm }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/clients', {
      path: {
        realm_name: realm || 'master',
      },
    }).queryOptions
  )
}

export const useGetClient = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
      path: {
        client_id: clientId!,
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}

export interface CreateClientMutate {
  realm: string
  payload: CreateClientSchema
}

export const useCreateClient = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/clients').mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['clients'],
      })
    },
  })
}

export const useUpdateClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('patch', '/realms/{realm_name}/clients/{client_id}')
      .mutationOptions,
    onSuccess: async (payload, variables) => {
      const client = await payload.json()

      const keys = window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
        path: {
          client_id: variables.path.client_id,
          realm_name: variables.path.realm_name,
        },
      }).queryKey

      toast.success(`Client ${client.data.name} was updated successfully`)
      queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useDeleteClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/clients/{client_id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (res) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/clients', {
        path: {
          realm_name: res.realm_name,
        },
      }).queryKey
      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useGetClientRoles = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/clients/{client_id}/roles', {
      path: {
        realm_name: realm!,
        client_id: clientId!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}
