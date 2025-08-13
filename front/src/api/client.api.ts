import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery, tanstackApi } from '.'
import { CreateClientSchema } from '@/pages/client/schemas/create-client.schema.ts'
import { toast } from 'sonner'

export const useGetClients = ({ realm }: BaseQuery) => {
  return useQuery(
    tanstackApi.get('/realms/{realm_name}/clients', {
      path: {
        realm_name: realm || 'master',
      },
    }).queryOptions
  )
}

export const useGetClient = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...tanstackApi.get('/realms/{realm_name}/clients/{client_id}', {
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
    ...tanstackApi.mutation('post', '/realms/{realm_name}/clients').mutationOptions,
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
    ...tanstackApi.mutation('patch', '/realms/{realm_name}/clients/{client_id}').mutationOptions,
    onSuccess: async (payload) => {
      const client = await payload.json()

      toast.success(`Client ${client.name} was updated successfully`)
      queryClient.invalidateQueries({
        queryKey: ['client', client.id],
      })
    },
  })
}

export const useDeleteClient = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...tanstackApi.mutation('delete', '/realms/{realm_name}/clients/{client_id}').mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['clients'],
      })
    },
  })
}

export const useGetClientRoles = ({ realm, clientId }: BaseQuery & { clientId?: string }) => {
  return useQuery({
    ...tanstackApi.get('/realms/{realm_name}/clients/{client_id}/roles', {
      path: {
        realm_name: realm!,
        client_id: clientId!,
      },
    }).queryOptions,
    enabled: !!clientId && !!realm,
  })
}
