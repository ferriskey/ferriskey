import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'

export const useGetWebhooks = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/webhooks', {
      path: {
        realm_name: realm,
      },
    }).queryOptions
  )
}

export const useCreateWebhook = () => {
  return useMutation(
    window.tanstackApi.mutation('post', '/realms/{realm_name}/webhooks', async (res) => res.json())
      .mutationOptions
  )
}

export const useDeleteWebhook = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/webhooks/{webhook_id}',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async (data) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/webhooks', {
        path: {
          realm_name: data.realm_name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}
