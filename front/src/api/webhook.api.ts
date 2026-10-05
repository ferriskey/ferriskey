import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type WebhooksQuery = NonNullable<Endpoints.get_Fetch_webhooks['parameters']['query']>

export type WebhooksFilter = Omit<WebhooksQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const WEBHOOK_FILTER_KEYS = [
  'name',
  'endpoint',
  'triggered',
  'has_subscribers',
  'secure_endpoint',
  'search',
  'created_from',
  'created_to',
] as const

export const webhooksKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/webhooks', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetWebhooks = ({
  realm = 'master',
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { query?: WebhooksQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/webhooks', {
      path: {
        realm_name: realm,
      },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled,
  })
}

export const useWebhookCount = ({
  realm,
  filter,
  enabled = true,
}: BaseQuery & { filter?: WebhooksFilter; enabled?: boolean }) => {
  const { data, isLoading } = useGetWebhooks({
    realm,
    query: { ...filter, limit: 1 },
    enabled,
  })
  return { count: data?.metadata.total ?? 0, isLoading }
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
      await queryClient.invalidateQueries({
        queryKey: webhooksKey(data.realm_name),
      })
    },
  })
}

export type WebhookDeliveriesQuery = NonNullable<
  Endpoints.get_Fetch_deliveries['parameters']['query']
>

export type WebhookDeliveriesFilter = Omit<
  WebhookDeliveriesQuery,
  'page' | 'limit' | 'order' | 'order_by'
>

export const DELIVERY_FILTER_KEYS = [
  'status',
  'event',
  'resource_id',
  'created_from',
  'created_to',
] as const

export const useGetWebhookDeliveries = ({
  realm = 'master',
  webhookId,
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { webhookId: string; query?: WebhookDeliveriesQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/webhooks/{webhook_id}/deliveries', {
      path: {
        realm_name: realm,
        webhook_id: webhookId,
      },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: enabled && Boolean(webhookId),
  })
}

export const useWebhookDeliveryCount = ({
  realm,
  webhookId,
  filter,
  enabled = true,
}: BaseQuery & { webhookId: string; filter?: WebhookDeliveriesFilter; enabled?: boolean }) => {
  const { data, isLoading } = useGetWebhookDeliveries({
    realm,
    webhookId,
    query: { ...filter, limit: 1 },
    enabled,
  })
  return { count: data?.metadata.total ?? 0, isLoading }
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
