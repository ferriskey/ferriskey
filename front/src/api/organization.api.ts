import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import { translate } from '@/lib/i18n'

export const useGetOrganizations = ({ realm }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/organizations', {
      path: { realm_name: realm ?? 'master' },
    }).queryOptions
  )
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
      const keys = window.tanstackApi.get('/realms/{realm_name}/organizations', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      toast.success(translate('common:toast.organization.created', { name: payload.name }))
      await queryClient.invalidateQueries({ queryKey: keys })
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
      queryClient.invalidateQueries({ queryKey: keys })
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
      const keys = window.tanstackApi.get('/realms/{realm_name}/organizations', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      toast.success(translate('common:toast.organization.deleted'))
      await queryClient.invalidateQueries({ queryKey: keys })
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

export const useGetOrganizationMembers = ({
  realm,
  organizationId,
}: BaseQuery & { organizationId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}/members', {
      path: { realm_name: realm!, organization_id: organizationId! },
    }).queryOptions,
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
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/members',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success(translate('common:toast.organization.member_removed'))
      await queryClient.invalidateQueries({ queryKey: keys })
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
      const orgMembersKeys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/members',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success(translate('common:toast.organization.user_added'))
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userOrgsKeys }),
        queryClient.invalidateQueries({ queryKey: orgMembersKeys }),
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
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
