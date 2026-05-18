import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Schemas } from './api.client'

// ---------- Legacy single-theme-per-realm hooks (kept until cleanup PR) ----------

export const useGetPortalTheme = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useUpdatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal/theme').mutationOptions,
    onSuccess: async (_, variables) => {
      const themeKeys = window.tanstackApi.get('/realms/{realm_name}/portal/theme', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      const loginSettingsKeys = window.tanstackApi.get('/realms/{name}/login-settings', {
        path: { name: variables.path.realm_name },
      }).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: themeKeys }),
        queryClient.invalidateQueries({ queryKey: loginSettingsKeys }),
      ])

      toast.success('Portal theme saved')
    },
  })
}

// ---------- Collection API ----------

export const useListPortalThemes = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetPortalThemeById = ({
  realm = 'master',
  themeId,
}: BaseQuery & { themeId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/themes/{theme_id}', {
      path: { realm_name: realm, theme_id: themeId },
    }).queryOptions,
    enabled: !!realm && !!themeId && themeId !== 'new',
  })
}

export const useCreatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/portal/themes').mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: listKey })
      toast.success('Portal theme created')
    },
  })
}

export const useUpdatePortalThemeMetadata = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal/themes/{theme_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
      toast.success('Portal theme updated')
    },
  })
}

export const useUpdatePortalThemePage = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/portal/themes/{theme_id}/pages/{page_type}',
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
      toast.success('Portal page saved')
    },
  })
}

export const useActivatePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/portal/themes/{theme_id}/activate',
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      await invalidateThemeQueries(queryClient, variables.path.realm_name, variables.path.theme_id)
      toast.success('Portal theme activated')
    },
  })
}

export const useDeletePortalTheme = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/portal/themes/{theme_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      await queryClient.invalidateQueries({ queryKey: listKey })
      toast.success('Portal theme deleted')
    },
  })
}

// ---------- Public renderer + introspection ----------

export const useGetActivePortalTheme = ({
  realm = 'master',
  pageType,
}: BaseQuery & { pageType: Schemas.PortalPageType }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/active', {
      path: { realm_name: realm },
      query: { page_type: pageType },
    }).queryOptions,
    enabled: !!realm && !!pageType,
  })
}

export const useGetPortalPageRequirements = ({ realm = 'master' }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/portal/page-requirements', {
      path: { realm_name: realm },
    }).queryOptions,
    enabled: !!realm,
  })
}

async function invalidateThemeQueries(
  queryClient: ReturnType<typeof useQueryClient>,
  realmName: string,
  themeId: string,
) {
  const listKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes', {
    path: { realm_name: realmName },
  }).queryKey
  const itemKey = window.tanstackApi.get('/realms/{realm_name}/portal/themes/{theme_id}', {
    path: { realm_name: realmName, theme_id: themeId },
  }).queryKey

  await Promise.all([
    queryClient.invalidateQueries({ queryKey: listKey }),
    queryClient.invalidateQueries({ queryKey: itemKey }),
  ])
}
