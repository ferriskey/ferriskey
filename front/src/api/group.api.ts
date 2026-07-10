import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { authStore } from '@/store/auth.store'

// Hierarchical groups scoped to an organization. These hooks use `fetch` directly (like
// usePublicPasswordPolicy) because the generated OpenAPI client does not yet expose the group
// endpoints; regenerate `api.client.ts` to migrate onto `window.tanstackApi`.

export interface Group {
  id: string
  organization_id: string
  parent_group_id: string | null
  name: string
  description: string | null
  created_at: string
  updated_at: string
}

export interface GroupNode extends Group {
  children: GroupNode[]
}

export interface GroupMember {
  id: string
  group_id: string
  user_id: string
  created_at: string
}

/** A member enriched with the user's identity, as returned by the paginated members endpoint. */
export interface GroupMemberDetail {
  id: string
  group_id: string
  user_id: string
  username: string
  email: string | null
  firstname: string | null
  lastname: string | null
  enabled: boolean
  created_at: string
}

export interface GroupMembersPage {
  data: GroupMemberDetail[]
  total: number
  limit: number
  offset: number
}

export interface GroupMembersParams {
  limit?: number
  offset?: number
  search?: string
}

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
    let message = `HTTP ${response.status}`
    try {
      const data = await response.json()
      message = data?.message ?? data?.errors?.[0]?.message ?? message
    } catch {
      // keep default message
    }
    throw new Error(message)
  }

  if (response.status === 204) {
    return undefined as T
  }
  return response.json() as Promise<T>
}

function groupsBase(realm: string, orgId: string): string {
  return `/realms/${encodeURIComponent(realm)}/organizations/${orgId}/groups`
}

const groupsKey = (realm?: string, orgId?: string) => ['org-groups', realm, orgId]

export function useGroups(realm?: string, orgId?: string) {
  return useQuery<GroupNode[]>({
    queryKey: groupsKey(realm, orgId),
    queryFn: () => request<GroupNode[]>('GET', groupsBase(realm!, orgId!)),
    enabled: !!realm && !!orgId,
  })
}

export function useCreateGroup(realm?: string, orgId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { name: string; description?: string; parent_group_id?: string }) =>
      request<Group>('POST', groupsBase(realm!, orgId!), body),
    onSuccess: () => qc.invalidateQueries({ queryKey: groupsKey(realm, orgId) }),
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
    onSuccess: () => qc.invalidateQueries({ queryKey: groupsKey(realm, orgId) }),
  })
}

export function useDeleteGroup(realm?: string, orgId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (groupId: string) =>
      request<void>('DELETE', `${groupsBase(realm!, orgId!)}/${groupId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: groupsKey(realm, orgId) }),
  })
}

const membersKey = (realm?: string, orgId?: string, groupId?: string) => [
  'org-group-members',
  realm,
  orgId,
  groupId,
]

export function useGroupMembers(
  realm?: string,
  orgId?: string,
  groupId?: string,
  params?: GroupMembersParams
) {
  const limit = params?.limit ?? 50
  const offset = params?.offset ?? 0
  const search = params?.search ?? ''

  return useQuery<GroupMembersPage>({
    queryKey: [...membersKey(realm, orgId, groupId), limit, offset, search],
    queryFn: () => {
      const qs = new URLSearchParams()
      qs.set('limit', String(limit))
      qs.set('offset', String(offset))
      if (search) qs.set('search', search)
      return request<GroupMembersPage>(
        'GET',
        `${groupsBase(realm!, orgId!)}/${groupId}/members?${qs.toString()}`
      )
    },
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
    onSuccess: () => qc.invalidateQueries({ queryKey: membersKey(realm, orgId, groupId) }),
  })
}

export function useRemoveGroupMember(realm?: string, orgId?: string, groupId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (userId: string) =>
      request<void>('DELETE', `${groupsBase(realm!, orgId!)}/${groupId}/members/${userId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: membersKey(realm, orgId, groupId) }),
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
