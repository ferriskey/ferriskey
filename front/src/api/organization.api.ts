import { useEffect, useMemo, useState } from 'react'
import { useMutation, useQueries, useQuery, useQueryClient, type UseQueryResult } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Endpoints, Schemas } from './api.client'
import { ID_BATCH, idBatches } from './id-batches'
import { translate } from '@/lib/i18n'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type OrganizationsQuery = NonNullable<
  Endpoints.get_List_organizations['parameters']['query']
>

export type OrganizationsFilter = Omit<OrganizationsQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const ORGANIZATION_FILTER_KEYS = [
  'search',
  'name',
  'alias',
  'domain',
  'description',
  'enabled',
  'has_domain',
  'created_from',
  'created_to',
] as const

export const ORGANIZATION_SEARCH_LIMIT = 20

export type OrganizationMembersQuery = NonNullable<
  Endpoints.get_List_members['parameters']['query']
>

export const ORGANIZATION_MEMBER_FILTER_KEYS = [
  'search',
  'username',
  'email',
  'enabled',
  'created_from',
  'created_to',
] as const

const SEARCH_DEBOUNCE_MS = 300

export const organizationsKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/organizations', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetOrganizations = ({
  realm,
  query,
  keepPrevious = false,
  enabled = true,
}: BaseQuery & { query?: OrganizationsQuery; enabled?: boolean } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations', {
      path: { realm_name: realm ?? 'master' },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled,
  })
}

export const useOrganizationCount = ({
  realm,
  filter,
  enabled = true,
}: BaseQuery & { filter?: OrganizationsFilter; enabled?: boolean }) => {
  const { data, isLoading } = useGetOrganizations({
    realm,
    query: { ...filter, limit: 1 },
    enabled,
  })
  return { count: data?.metadata.total ?? 0, isLoading }
}

const combineOrganizations = (results: UseQueryResult<Schemas.Paginated_Organization>[]) => ({
  organizations: results.flatMap((result) => result.data?.data ?? []),
  isLoading: results.some((result) => result.isLoading),
  isError: results.some((result) => result.isError),
})

export const useOrganizationsByIds = ({ realm, ids }: BaseQuery & { ids: readonly string[] }) => {
  const batches = useMemo(() => idBatches(ids), [ids])
  return useQueries({
    queries: batches.map((batch) => ({
      ...window.tanstackApi.get('/realms/{realm_name}/organizations', {
        path: { realm_name: realm ?? 'master' },
        query: { ids: batch, limit: ID_BATCH },
      }).queryOptions,
    })),
    combine: combineOrganizations,
  })
}

export const useOrganizationSearch = ({
  realm,
  filter,
  enabled = true,
}: BaseQuery & { filter?: OrganizationsFilter; enabled?: boolean }) => {
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const { data, isLoading } = useGetOrganizations({
    realm,
    query: {
      ...filter,
      search: debounced || undefined,
      order_by: 'name',
      order: 'asc',
      limit: ORGANIZATION_SEARCH_LIMIT,
    },
    enabled,
  })

  return {
    search,
    setSearch,
    organizations: data?.data ?? [],
    isLoading,
  }
}

export const useGetOrganization = ({
  realm,
  organizationId,
}: BaseQuery & { organizationId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}', {
      path: { realm_name: realm!, organization_id: organizationId! },
    }).queryOptions,
    enabled: !!realm && !!organizationId,
  })
}

export const useCreateOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/organizations').mutationOptions,
    onSuccess: async (payload, variables) => {
      toast.success(translate('common:toast.organization.created', { name: payload.name }))
      await queryClient.invalidateQueries({ queryKey: organizationsKey(variables.path.realm_name) })
    },
  })
}

export const useUpdateOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/organizations/{organization_id}')
      .mutationOptions,
    onSuccess: async (payload, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success(translate('common:toast.organization.updated', { name: payload.name }))
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: keys }),
        queryClient.invalidateQueries({ queryKey: organizationsKey(variables.path.realm_name) }),
      ])
    },
  })
}

export const useDeleteOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      toast.success(translate('common:toast.organization.deleted'))
      await queryClient.invalidateQueries({ queryKey: organizationsKey(variables.path.realm_name) })
    },
  })
}

export const useGetOrganizationAttributes = ({
  realm,
  organizationId,
}: BaseQuery & { organizationId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/organizations/{organization_id}/attributes',
      {
        path: { realm_name: realm!, organization_id: organizationId! },
      }
    ).queryOptions,
    select: (response) => response.data,
    enabled: !!realm && !!organizationId,
  })
}

export const useUpsertOrganizationAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/organizations/{organization_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/attributes',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useDeleteOrganizationAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/attributes',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success(translate('common:toast.attribute.deleted'))
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

const organizationMembersKey = (realm: string, organizationId: string) =>
  window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/members', {
    path: { realm_name: realm, organization_id: organizationId },
    query: {},
  }).queryKey

const realmUsersKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/users', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetOrganizationMembers = ({
  realm,
  organizationId,
  query,
  keepPrevious = false,
}: BaseQuery & { organizationId?: string; query?: OrganizationMembersQuery } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/members', {
      path: { realm_name: realm!, organization_id: organizationId! },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: !!realm && !!organizationId,
  })
}

export const useRemoveOrganizationMember = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}/members/{user_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      toast.success(translate('common:toast.organization.member_removed'))
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: organizationMembersKey(
            variables.path.realm_name,
            variables.path.organization_id
          ),
        }),
        queryClient.invalidateQueries({ queryKey: realmUsersKey(variables.path.realm_name) }),
      ])
    },
  })
}

export const useGetUserOrganizations = ({
  realm,
  userId,
}: BaseQuery & { userId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/organizations', {
      path: { realm_name: realm!, user_id: userId! },
    }).queryOptions,
    enabled: !!realm && !!userId,
  })
}

export const useAddUserToOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/organizations/{organization_id}/members'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const userOrgsKeys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/organizations', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.body.user_id,
        },
      }).queryKey
      toast.success(translate('common:toast.organization.user_added'))
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userOrgsKeys }),
        queryClient.invalidateQueries({
          queryKey: organizationMembersKey(
            variables.path.realm_name,
            variables.path.organization_id
          ),
        }),
        queryClient.invalidateQueries({ queryKey: realmUsersKey(variables.path.realm_name) }),
        queryClient.invalidateQueries({ queryKey: organizationsKey(variables.path.realm_name) }),
      ])
    },
  })
}

export const useRemoveUserFromOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}/members/{user_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/organizations', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      toast.success(translate('common:toast.organization.user_removed'))
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: keys }),
        queryClient.invalidateQueries({
          queryKey: organizationMembersKey(
            variables.path.realm_name,
            variables.path.organization_id
          ),
        }),
        queryClient.invalidateQueries({ queryKey: realmUsersKey(variables.path.realm_name) }),
        queryClient.invalidateQueries({ queryKey: organizationsKey(variables.path.realm_name) }),
      ])
    },
  })
}
