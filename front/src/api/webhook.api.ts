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

export const useGetWebhook = ({ realm = 'master', webhookId }: BaseQuery & { webhookId: string }) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/webhooks/{webhook_id}', {
      path: {
        realm_name: realm,
        webhook_id: webhookId,
      },
    }).queryOptions
  )
}

export const useCreateWebhook = () => {
  return useMutation(
    window.tanstackApi.mutation('post', '/realms/{realm_name}/webhooks').mutationOptions
  )
}

export const useUpdateWebhook = () => {
  return useMutation(
    window.tanstackApi.mutation('put', '/realms/{realm_name}/webhooks/{webhook_id}').mutationOptions
  )
}

export const useDeleteWebhook = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/webhooks/{webhook_id}')
      .mutationOptions,
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

export const useGetWebhookDeliveries = ({
  realm = 'master',
  webhookId,
  status,
  limit,
  offset,
}: BaseQuery & { webhookId: string; status?: string; limit: number; offset: number }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/webhooks/{webhook_id}/deliveries', {
      path: {
        realm_name: realm,
        webhook_id: webhookId,
      },
      query: {
        status: status || undefined,
        limit,
        offset,
      },
    }).queryOptions,
    enabled: Boolean(webhookId),
  })
}

export const useGetWebhookDelivery = ({
  realm = 'master',
  webhookId,
  deliveryId,
}: BaseQuery & { webhookId: string; deliveryId: string | null }) => {
  return useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}',
      {
        path: {
          realm_name: realm,
          webhook_id: webhookId,
          delivery_id: deliveryId ?? '',
        },
      }
    ).queryOptions,
    enabled: Boolean(deliveryId),
  })
}

export const useRetryWebhookDelivery = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}/retry'
    ).mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        predicate: (query) => {
          const key = query.queryKey[0] as { _id?: string } | undefined
          return key?._id === '/realms/{realm_name}/webhooks/{webhook_id}/deliveries'
        },
      })
    },
  })
}

export const useRotateWebhookSecret = () => {
  return useMutation(
    window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/webhooks/{webhook_id}/secret/rotate'
    ).mutationOptions
  )
}
