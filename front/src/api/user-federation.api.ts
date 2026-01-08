import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

export const useCreateUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/federation/providers',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, params) => {
      const queryKeys = window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
        path: {
          realm_name: params.path.realm_name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: queryKeys,
      })
    },
  })
}

export const useGetUserFederations = (realm_name: string) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
      path: {
        realm_name,
      },
    }).queryOptions,
  })
}

export const useDeleteUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/federation/providers/{id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, params) => {
      const queryKeys = window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
        path: {
          realm_name: params.path.realm_name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: queryKeys,
      })
    },
  })
}
