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

export const useGetUserFederation = (realm_name: string, id: string) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/federation/providers/{id}', {
      path: {
        realm_name,
        id,
      },
    }).queryOptions,
    enabled: !!realm_name && !!id,
  })
}

export const useUpdateUserFederation = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/federation/providers/{id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (_, params) => {
      const listQueryKeys = window.tanstackApi.get('/realms/{realm_name}/federation/providers', {
        path: {
          realm_name: params.path.realm_name,
        },
      }).queryKey

      const detailQueryKeys = window.tanstackApi.get(
        '/realms/{realm_name}/federation/providers/{id}',
        {
          path: {
            realm_name: params.path.realm_name,
            id: params.path.id,
          },
        }
      ).queryKey

      await queryClient.invalidateQueries({
        queryKey: listQueryKeys,
      })

      await queryClient.invalidateQueries({
        queryKey: detailQueryKeys,
      })
    },
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

export const useTestUserFederationConnection = () => {
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/federation/providers/{id}/test-connection',
      async (res) => res.json()
    ).mutationOptions,
  })
}
