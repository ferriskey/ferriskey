import { useMemo } from 'react'
import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
  type UseQueryResult,
} from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Endpoints, Schemas } from './api.client'
import { ID_BATCH, idBatches } from './id-batches'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

export type PortalLayoutsQuery = NonNullable<
  Endpoints.get_List_layouts['parameters']['query']
>

export type PortalLayoutsFilter = Omit<PortalLayoutsQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const PORTAL_LAYOUT_FILTER_KEYS = [
  'search',
  'name',
  'is_default',
  'in_use',
  'created_from',
  'created_to',
] as const

export const PORTAL_LAYOUT_SEARCH_LIMIT = 20

export const portalLayoutsKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetPortalLayouts = ({
  realm = 'master',
  query,
  enabled = true,
}: BaseQuery & { query?: PortalLayoutsQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
      path: { realm_name: realm },
      query: query ?? {},
    }).queryOptions,
    enabled: enabled && !!realm,
  })
}

export const usePortalLayoutCount = ({
  realm = 'master',
  filter,
}: BaseQuery & { filter?: PortalLayoutsFilter }) => {
  const { data, isLoading } = useGetPortalLayouts({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}

const combineLayouts = (results: UseQueryResult<Schemas.Paginated_PortalLayoutListItem>[]) => ({
  layouts: results.flatMap((result) => result.data?.data ?? []),
  isLoading: results.some((result) => result.isLoading),
})

export const usePortalLayoutsByIds = ({
  realm = 'master',
  ids,
}: BaseQuery & { ids: readonly string[] }) => {
  const batches = useMemo(() => idBatches(ids), [ids])
  return useQueries({
    queries: batches.map((batch) => ({
      ...window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
        path: { realm_name: realm },
        query: { ids: batch, limit: ID_BATCH },
      }).queryOptions,
    })),
    combine: combineLayouts,
  })
}

export const useGetPortalLayout = ({
  realm = 'master',
  layoutId,
}: BaseQuery & { layoutId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal-layouts/{layout_id}', {
      path: { realm_name: realm, layout_id: layoutId },
    }).queryOptions,
    enabled: !!layoutId && layoutId !== 'new',
  })
}

export const useGetPublicDefaultPortalLayout = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal-layouts/public/default', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetPublicPortalLayout = ({
  realm = 'master',
  layoutId,
}: BaseQuery & { layoutId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal-layouts/public/{layout_id}', {
      path: { realm_name: realm, layout_id: layoutId },
    }).queryOptions,
    enabled: !!realm && !!layoutId,
  })
}

export const useCreatePortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/portal-layouts').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = portalLayoutsKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.portal_layout.created'))
    },
  })
}

export const useUpdatePortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal-layouts/{layout_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = portalLayoutsKey(variables.path.realm_name)
      const itemKey = window.tanstackApi.get('/realms/{realm_name}/portal-layouts/{layout_id}', {
        path: { realm_name: variables.path.realm_name, layout_id: variables.path.layout_id },
      }).queryKey
      const publicKey = window.tanstackApi.get(
        '/realms/{realm_name}/portal-layouts/public/default',
        { path: { realm_name: variables.path.realm_name } },
      ).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: listKey }),
        queryClient.invalidateQueries({ queryKey: itemKey }),
        queryClient.invalidateQueries({ queryKey: publicKey }),
      ])
      toast.success(translate('common:toast.portal_layout.saved'))
    },
  })
}

export const useDeletePortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/portal-layouts/{layout_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = portalLayoutsKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: listKey })
      toast.success(translate('common:toast.portal_layout.deleted'))
    },
  })
}

export const useSetDefaultPortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/portal-layouts/{layout_id}/default',
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = portalLayoutsKey(variables.path.realm_name)
      const publicKey = window.tanstackApi.get(
        '/realms/{realm_name}/portal-layouts/public/default',
        { path: { realm_name: variables.path.realm_name } },
      ).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: listKey }),
        queryClient.invalidateQueries({ queryKey: publicKey }),
      ])
      toast.success(translate('common:toast.portal_layout.default_updated'))
    },
  })
}

export const useImportPortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/portal-layouts/import')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = portalLayoutsKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.portal_layout.imported'))
    },
    // The server refuses a file that belongs to the other builder or that uses
    // a format version it cannot read; its message says which, so surface it.
    onError: (error) => {
      toast.error(translate('common:toast.import_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}
