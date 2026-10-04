import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'

export type UserRealmsQuery = NonNullable<Endpoints.get_Get_user_realms['parameters']['query']>

export const REALM_SEARCH_LIMIT = 20

const SEARCH_DEBOUNCE_MS = 300

export const useGetUserRealmsQuery = ({
  realm,
  query,
  enabled = true,
}: BaseQuery & { query?: UserRealmsQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
      path: {
        realm_name: realm || 'master',
      },
      query: query ?? {},
    }).queryOptions,
    enabled,
  })
}

export const useUserRealmSearch = ({
  realm,
  enabled = true,
}: BaseQuery & { enabled?: boolean }) => {
  const [search, setSearch] = useState('')
  const [debounced, setDebounced] = useState('')

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timer)
  }, [search])

  const { data, isLoading } = useGetUserRealmsQuery({
    realm,
    query: {
      name: debounced || undefined,
      order_by: 'name',
      order: 'asc',
      limit: REALM_SEARCH_LIMIT,
    },
    enabled,
  })

  return {
    search,
    setSearch,
    realms: data?.data ?? [],
    isLoading,
  }
}

export const useCreateRealm = ({ realm }: BaseQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms').mutationOptions,

    onSuccess: async () => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
        path: {
          realm_name: realm || 'master',
        },
        query: {},
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}

export const useGetLoginSettings = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{name}/login-settings', {
      path: {
        name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetRealm = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{name}', {
      path: {
        name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useDeleteRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{name}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
        path: {
          realm_name: variables.path.name,
        },
        query: {},
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useUpdateRealm = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{name}').mutationOptions,
    onSuccess: async (_, variables) => {
      const userRealmsKeys = window.tanstackApi.get('/realms/{realm_name}/users/@me/realms', {
        path: {
          realm_name: variables.path.name,
        },
        query: {},
      }).queryKey

      const realmKeys = window.tanstackApi.get('/realms/{name}', {
        path: {
          name: variables.path.name,
        },
      }).queryKey

      const loginKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: {
          name: variables.path.name,
        },
      }).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userRealmsKeys }),
        queryClient.invalidateQueries({ queryKey: realmKeys }),
        queryClient.invalidateQueries({ queryKey: loginKeys }),
      ])
    },
  })
}

export const useUpdateRealmSettings = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{name}/settings').mutationOptions,
    onSuccess: async (res) => {
      const loginKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: {
          name: res.data.name,
        },
      }).queryKey

      const realmKeys = window.tanstackApi.get('/realms/{name}', {
        path: {
          name: res.data.name,
        },
      }).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [...loginKeys] }),
        queryClient.invalidateQueries({ queryKey: [...realmKeys] }),
      ])
    },
  })
}

export const useGetRealmPasswordPolicy = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/password-policy', {
      path: {
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdateRealmPasswordPolicy = () => {
  const queryClient = useQueryClient()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/password-policy').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/password-policy', {
        path: {
          realm_name: variables.path.realm_name,
        },
      }).queryKey

      await queryClient.invalidateQueries({
        queryKey: keys,
      })
    },
  })
}
