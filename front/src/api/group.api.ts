import { useMemo } from 'react'
import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
  type QueryClient,
  type UseQueryResult,
} from '@tanstack/react-query'
import { authStore } from '@/store/auth.store'
import { errorMessageFromBody, type ApiRequestError } from '@/lib/api-error'
import type { Endpoints, Schemas } from './api.client'
import { ID_BATCH, idBatches } from './id-batches'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

export type Group = Schemas.Group

export type GroupListItem = Schemas.GroupListItem

export type GroupsQuery = NonNullable<Endpoints.get_List_groups['parameters']['query']>

export const GROUP_FILTER_KEYS = [
  'search',
  'parent_group_id',
  'is_root',
  'created_from',
  'created_to',
] as const

export const GROUP_SEARCH_LIMIT = 20

export interface GroupMember {
  id: string
  group_id: string
  user_id: string
  created_at: string
}

export type GroupMemberDetail = Schemas.GroupMemberDetail

export type GroupMembersQuery = NonNullable<
  Endpoints.get_List_group_members['parameters']['query']
>

export const GROUP_MEMBER_FILTER_KEYS = [
  'search',
  'enabled',
  'created_from',
  'created_to',
] as const

export interface GroupAttribute {
  id: string
  group_id: string
  key: string
  value: string
  created_at: string
}

export interface GroupRole {
  id: string
  name: string
  description?: string | null
  client_id?: string | null
}

function apiBase(): string {
  return (window.apiUrl ?? '').replace(/\/$/, '')
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const token = authStore.getState().accessToken
  if (token) {
    headers.Authorization = `Bearer ${token}`
  }

  const response = await fetch(`${apiBase()}${path}`, {
    method,
    headers,
    credentials: 'include',
    body: body === undefined ? undefined : JSON.stringify(body),
  })

  if (!response.ok) {
    let errorBody: Record<string, unknown> | undefined
    try {
      errorBody = await response.json()
    } catch {
      errorBody = undefined
    }

    const error: ApiRequestError = new Error(
      errorMessageFromBody(errorBody) ?? `HTTP ${response.status}`
    )
    error.status = response.status
    error.data = errorBody
    throw error
  }

  if (response.status === 204) {
    return undefined as T
  }
  return response.json() as Promise<T>
}

function groupsBase(realm: string, orgId: string): string {
  return `/realms/${encodeURIComponent(realm)}/organizations/${orgId}/groups`
}

const groupsPath = (realm?: string, orgId?: string) => ({
  realm_name: realm ?? 'master',
  organization_id: orgId ?? '',
})

export const groupsKey = (realm?: string, orgId?: string) =>
  window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/groups', {
    path: groupsPath(realm, orgId),
    query: {},
  }).queryKey

const groupKey = (realm?: string, orgId?: string, groupId?: string) =>
  window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}', {
    path: { ...groupsPath(realm, orgId), group_id: groupId ?? '' },
  }).queryKey

const groupDetailsKey = (realm?: string, orgId?: string) => [
  {
    _id: '/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}',
    path: groupsPath(realm, orgId),
  },
]

const refreshGroups = (qc: QueryClient, realm?: string, orgId?: string, groupId?: string) =>
  Promise.all([
    qc.invalidateQueries({ queryKey: groupsKey(realm, orgId) }),
    groupId ? qc.invalidateQueries({ queryKey: groupKey(realm, orgId, groupId) }) : undefined,
  ])

export function useGroups({
  realm,
  orgId,
  query,
  keepPrevious = false,
  enabled = true,
}: {
  realm?: string
  orgId?: string
  query?: GroupsQuery
  enabled?: boolean
} & PagedQueryOptions) {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/groups', {
      path: groupsPath(realm, orgId),
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: enabled && !!realm && !!orgId,
  })
}

const combineGroups = (results: UseQueryResult<Schemas.Paginated_GroupListItem>[]) => ({
  groups: results.flatMap((result) => result.data?.data ?? []),
  isLoading: results.some((result) => result.isLoading),
})

export function useGroupsByIds({
  realm,
  orgId,
  ids,
}: {
  realm?: string
  orgId?: string
  ids: readonly string[]
}) {
  const batches = useMemo(() => idBatches(ids), [ids])
  return useQueries({
    queries: batches.map((batch) => ({
      ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/groups', {
        path: groupsPath(realm, orgId),
        query: { ids: batch, limit: ID_BATCH },
      }).queryOptions,
      enabled: !!realm && !!orgId,
    })),
    combine: combineGroups,
  })
}

export function useGroup({
  realm,
  orgId,
  groupId,
}: {
  realm?: string
  orgId?: string
  groupId?: string
}) {
  return useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}',
      { path: { ...groupsPath(realm, orgId), group_id: groupId ?? '' } }
    ).queryOptions,
    enabled: !!realm && !!orgId && !!groupId,
  })
}

export function useCreateGroup(realm?: string, orgId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { name: string; description?: string; parent_group_id?: string }) =>
      request<Group>('POST', groupsBase(realm!, orgId!), body),
    onSuccess: () => refreshGroups(qc, realm, orgId),
  })
}

export function useUpdateGroup(realm?: string, orgId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      groupId,
      ...body
    }: {
      groupId: string
      name?: string
      description?: string
      parent_group_id?: string
    }) => request<Group>('PUT', `${groupsBase(realm!, orgId!)}/${groupId}`, body),
    onSuccess: (_, { groupId }) => refreshGroups(qc, realm, orgId, groupId),
  })
}

export function useDeleteGroup(realm?: string, orgId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (groupId: string) =>
      request<void>('DELETE', `${groupsBase(realm!, orgId!)}/${groupId}`),
    onSuccess: (_, groupId) => {
      qc.removeQueries({ queryKey: groupKey(realm, orgId, groupId) })
      return Promise.all([
        qc.invalidateQueries({ queryKey: groupsKey(realm, orgId) }),
        qc.invalidateQueries({ queryKey: groupDetailsKey(realm, orgId) }),
      ])
    },
  })
}

const membersPath = (realm?: string, orgId?: string, groupId?: string) => ({
  ...groupsPath(realm, orgId),
  group_id: groupId ?? '',
})

const membersKey = (realm?: string, orgId?: string, groupId?: string) =>
  window.tanstackApi.get(
    '/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members',
    { path: membersPath(realm, orgId, groupId), query: {} }
  ).queryKey

const realmUsersKey = (realm?: string) =>
  window.tanstackApi.get('/realms/{realm_name}/users', {
    path: { realm_name: realm || 'master' },
    query: {},
  }).queryKey

const refreshMembers = (qc: QueryClient, realm?: string, orgId?: string, groupId?: string) =>
  Promise.all([
    qc.invalidateQueries({ queryKey: membersKey(realm, orgId, groupId) }),
    qc.invalidateQueries({ queryKey: realmUsersKey(realm) }),
  ])

export function useGroupMembers({
  realm,
  orgId,
  groupId,
  query,
  keepPrevious = false,
}: {
  realm?: string
  orgId?: string
  groupId?: string
  query?: GroupMembersQuery
} & PagedQueryOptions) {
  return useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members',
      { path: membersPath(realm, orgId, groupId), query: query ?? {} }
    ).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: !!realm && !!orgId && !!groupId,
  })
}

export function useAddGroupMember(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (userId: string) =>
      request<GroupMember>('POST', `${groupsBase(realm!, orgId!)}/${groupId}/members`, {
        user_id: userId,
      }),
    onSuccess: () => refreshMembers(qc, realm, orgId, groupId),
  })
}

export function useRemoveGroupMember(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (userId: string) =>
      request<void>('DELETE', `${groupsBase(realm!, orgId!)}/${groupId}/members/${userId}`),
    onSuccess: () => refreshMembers(qc, realm, orgId, groupId),
  })
}

const rolesKey = (realm?: string, orgId?: string, groupId?: string) => [
  'org-group-roles',
  realm,
  orgId,
  groupId,
]

export function useGroupRoles(realm?: string, orgId?: string, groupId?: string) {
  return useQuery<GroupRole[]>({
    queryKey: rolesKey(realm, orgId, groupId),
    queryFn: () => request<GroupRole[]>('GET', `${groupsBase(realm!, orgId!)}/${groupId}/roles`),
    enabled: !!realm && !!orgId && !!groupId,
  })
}

export function useAssignGroupRole(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (roleId: string) =>
      request<void>('POST', `${groupsBase(realm!, orgId!)}/${groupId}/roles`, { role_id: roleId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: rolesKey(realm, orgId, groupId) }),
  })
}

export function useRevokeGroupRole(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (roleId: string) =>
      request<void>('DELETE', `${groupsBase(realm!, orgId!)}/${groupId}/roles/${roleId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: rolesKey(realm, orgId, groupId) }),
  })
}

const attrsKey = (realm?: string, orgId?: string, groupId?: string) => [
  'org-group-attributes',
  realm,
  orgId,
  groupId,
]

export function useGroupAttributes(realm?: string, orgId?: string, groupId?: string) {
  return useQuery<GroupAttribute[]>({
    queryKey: attrsKey(realm, orgId, groupId),
    queryFn: () =>
      request<GroupAttribute[]>('GET', `${groupsBase(realm!, orgId!)}/${groupId}/attributes`),
    enabled: !!realm && !!orgId && !!groupId,
  })
}

export function useUpsertGroupAttribute(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) =>
      request<GroupAttribute>(
        'PUT',
        `${groupsBase(realm!, orgId!)}/${groupId}/attributes/${encodeURIComponent(key)}`,
        { value }
      ),
    onSuccess: () => qc.invalidateQueries({ queryKey: attrsKey(realm, orgId, groupId) }),
  })
}

export function useDeleteGroupAttribute(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (key: string) =>
      request<void>(
        'DELETE',
        `${groupsBase(realm!, orgId!)}/${groupId}/attributes/${encodeURIComponent(key)}`
      ),
    onSuccess: () => qc.invalidateQueries({ queryKey: attrsKey(realm, orgId, groupId) }),
  })
}
