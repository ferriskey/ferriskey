import { useEffect, useMemo, useState } from 'react'
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
import { USER_IDS_BATCH, idBatches } from './user-ids'
import { translate } from '@/lib/i18n'

export interface UserMutateContract<T> {
  realm?: string
  userId?: string
  payload: T
}

export interface GetUserQueryParams {
  realm?: string
  userId?: string
}

export type UsersQuery = NonNullable<Endpoints.get_Get_users['parameters']['query']>

export type UsersFilter = Omit<UsersQuery, 'page' | 'limit' | 'order' | 'order_by'>

export const USER_SEARCH_LIMIT = 20

const SEARCH_DEBOUNCE_MS = 300

export const useGetUsers = ({
  realm,
  query,
  enabled = true,
}: BaseQuery & { query?: UsersQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users', {
      path: {
        realm_name: realm || 'master',
      },
      query: query ?? {},
    }).queryOptions,
    enabled,
  })
}

export const useUserCount = ({ realm, filter }: BaseQuery & { filter?: UsersFilter }) => {
  const { data, isLoading } = useGetUsers({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}

const combineUsers = (results: UseQueryResult<Schemas.Paginated_User>[]) => ({
  users: results.flatMap((result) => result.data?.data ?? []),
  isLoading: results.some((result) => result.isLoading),
})

export const useUsersByIds = ({ realm, ids }: BaseQuery & { ids: readonly string[] }) => {
  const batches = useMemo(() => idBatches(ids), [ids])
  return useQueries({
    queries: batches.map((batch) => ({
      ...window.tanstackApi.get('/realms/{realm_name}/users', {
        path: { realm_name: realm || 'master' },
        query: { ids: batch, limit: USER_IDS_BATCH },
      }).queryOptions,
    })),
    combine: combineUsers,
  })
}

export const useUserSearch = ({ realm, filter }: BaseQuery & { filter?: UsersFilter }) => {
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const { data, isLoading } = useGetUsers({
    realm,
    query: { ...filter, username: debounced || undefined, limit: USER_SEARCH_LIMIT },
  })

  return {
    search,
    setSearch,
    users: data?.data ?? [],
    isLoading,
  }
}

export const useGetUser = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}

export const useGetOwnProfile = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/me', {
      path: {
        realm_name: realm || 'master',
      },
    }).queryOptions,
  })
}

export const useUpdateOwnProfile = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/me').mutationOptions,
    onSuccess: (_res, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/me', {
        path: {
          realm_name: variables.path.realm_name,
        },
      }).queryKey
      queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useGetUserSessions = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/sessions', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}

export const useRevokeUserSession = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/users/{user_id}/sessions/{session_id}'
    ).mutationOptions,
    onSuccess: (_res, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/sessions', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useGetUserCredentials = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/credentials', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}

export const useCreateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users').mutationOptions,
    onSuccess: async (res) => {
      const queryKeys = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name: res.data.realm!.name,
        },
        query: {},
      }).queryKey

      console.log(queryKeys)
      await queryClient.invalidateQueries({
        queryKey: [...queryKeys],
      })
    },
  })
}

export const useUpdateUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}').mutationOptions,
    onSuccess: (_res, variables) => {
      const { realm_name, user_id } = variables.path
      const userDetailKey = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}', {
        path: {
          realm_name,
          user_id,
        },
      }).queryKey
      const usersListKey = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name,
        },
        query: {},
      }).queryKey
      queryClient.invalidateQueries({ queryKey: userDetailKey })
      queryClient.invalidateQueries({ queryKey: usersListKey })
    },
  })
}

export const useBulkDeleteUser = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/bulk').mutationOptions,
    onSuccess: async (res) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users', {
        path: {
          realm_name: res.realm_name,
        },
        query: {},
      }).queryKey
      queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useResetUserPassword = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}/reset-password')
      .mutationOptions,
    onSuccess: async (res) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/credentials', {
        path: {
          realm_name: res.realm_name,
          user_id: res.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useGetUserRoles = ({ realm, userId }: BaseQuery & { userId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
      path: {
        realm_name: realm!,
        user_id: userId!,
      },
    }).queryOptions,
    enabled: !!realm && !!userId,
  })
}

export const useAssignUserRole = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/{user_id}/roles/{role_id}')
      .mutationOptions,
    onSuccess: async (data) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/roles', {
        path: {
          realm_name: data.realm_name,
          user_id: data.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

// ─── User Attributes ──────────────────────────────────────────────────────────

export const useGetUserAttributes = ({
  realm,
  userId,
}: BaseQuery & { userId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
      path: { realm_name: realm!, user_id: userId! },
    }).queryOptions,
    select: (response) => response.data,
    enabled: !!realm && !!userId,
  })
}

export const useSetUserAttributes = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/{user_id}/attributes')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useDeleteUserAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/users/{user_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/{user_id}/attributes', {
        path: {
          realm_name: variables.path.realm_name,
          user_id: variables.path.user_id,
        },
      }).queryKey
      toast.success(translate('common:toast.attribute.deleted'))
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
