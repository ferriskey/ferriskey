import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { authStore } from '@/store/auth.store'

// Roles scoped to an organization member (Discord-style member roles). These hooks use `fetch`
// directly (like group.api.ts) for consistency with the sibling organization/group endpoints.

/** A role as returned by the member-roles endpoint (subset of the backend `Role`). */
export interface MemberRole {
  id: string
  name: string
  description?: string | null
  client_id?: string | null
}

function apiBase(): string {
  return (window.apiUrl ?? '').replace(/\/$/, '')
}

function memberRolesBase(realm: string, orgId: string, userId: string): string {
  return `/realms/${encodeURIComponent(realm)}/organizations/${orgId}/members/${userId}/roles`
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

const memberRolesKey = (realm?: string, orgId?: string, userId?: string) => [
  'organization-member-roles',
  realm,
  orgId,
  userId,
]

export function useOrganizationMemberRoles(realm?: string, orgId?: string, userId?: string) {
  return useQuery<MemberRole[]>({
    queryKey: memberRolesKey(realm, orgId, userId),
    queryFn: () => request<MemberRole[]>('GET', memberRolesBase(realm!, orgId!, userId!)),
    enabled: !!realm && !!orgId && !!userId,
  })
}

export function useAssignOrganizationMemberRole(realm?: string, orgId?: string, userId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (roleId: string) =>
      request<void>('POST', memberRolesBase(realm!, orgId!, userId!), { role_id: roleId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: memberRolesKey(realm, orgId, userId) }),
  })
}

export function useRevokeOrganizationMemberRole(realm?: string, orgId?: string, userId?: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (roleId: string) =>
      request<void>('DELETE', `${memberRolesBase(realm!, orgId!, userId!)}/${roleId}`),
    onSuccess: () => qc.invalidateQueries({ queryKey: memberRolesKey(realm, orgId, userId) }),
  })
}
