import { useMutation, useQueryClient } from '@tanstack/react-query'

export const useAssignUserRole = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/{user_id}/roles/{role_id}')
      .mutationOptions,
    onSuccess: async (_res, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useUnassignUserRole = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/{user_id}/roles/{role_id}')
      .mutationOptions,
    onSuccess: async (_res, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
