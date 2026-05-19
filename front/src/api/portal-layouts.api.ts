import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetPortalLayouts = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
      path: { realm_name: realm },
    }).queryOptions,
  )
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
      const keys = window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('Portal layout created successfully')
    },
  })
}

export const useUpdatePortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/portal-layouts/{layout_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
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
      toast.success('Portal layout saved successfully')
    },
  })
}

export const useDeletePortalLayout = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/portal-layouts/{layout_id}')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: listKey })
      toast.success('Portal layout deleted successfully')
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
      const listKey = window.tanstackApi.get('/realms/{realm_name}/portal-layouts', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      const publicKey = window.tanstackApi.get(
        '/realms/{realm_name}/portal-layouts/public/default',
        { path: { realm_name: variables.path.realm_name } },
      ).queryKey

      await Promise.all([
        queryClient.invalidateQueries({ queryKey: listKey }),
        queryClient.invalidateQueries({ queryKey: publicKey }),
      ])
      toast.success('Default portal layout updated')
    },
  })
}
